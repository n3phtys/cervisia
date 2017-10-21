extern crate gio;
extern crate glib;
extern crate gtk;
extern crate rand;

use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;

use gtk::{Builder, Button, MessageDialog};

use gio::prelude::*;
use gtk::{AboutDialog, AboutDialogExt, BoxExt, ContainerExt, DialogExt, GtkApplicationExt,
          Inhibit, LabelExt, SwitchExt, ToVariant, WidgetExt};
use std::env;


extern crate blrustix;
#[macro_use]
extern crate closet;

extern crate suffix;

#[macro_use]
extern crate lazy_static;

extern crate time;

extern crate chrono;


pub mod glade_builders;
pub mod input_handling;
pub mod static_variables;
pub mod cervisia_utilities;


use blrustix::*;
use blrustix::datastore::*;
use blrustix::persistencer::*;
use blrustix::rustix_backend;
use blrustix::rustix_backend::*;
use blrustix::datastore::Purchase::SimplePurchase;
use gtk::Adjustment;
use gtk::ScrolledWindow;
use rand::{Rng, SeedableRng, StdRng};
use std::borrow::*;
use std::cell::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::*;
use std::ops::DerefMut;
use std::rc::Rc;
use suffix::KDTree;
use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use glade_builders::*;
use cervisia_utilities::*;
use input_handling::*;
use static_variables::*;


use chrono::prelude::*;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

//TODO: remove old connect handlers before setting new ones, if not will lead to multi click handling
//TODO: both in user buttons and drink buttons
//TODO: alternative: build code handling into builder-functions, and never set new connect signals - instead store user and item id globally and go with those values




use glade_builders::QuickmenuGtkComponents;
use glade_builders::UserWindowGtkComponents;
use glade_builders::build_from_glade;
use glade_builders::build_quickmenu;

lazy_static! {
    static ref GLOBAL_BACKEND:
        Mutex<rustix_backend::RustixBackend<persistencer::TransientPersister>> =
        Mutex::new(blrustix::build_transient_backend_with(
            NUMBER_OF_USERS_PER_PAGE , NUMBER_OF_USERS_PER_PAGE
            ));
    static ref GLOBAL_QUICKMENU: Mutex<QuickmenuGtkComponents> = Mutex::new(build_quickmenu());
    static ref GLOBAL_USERWINDOW: Mutex<UserWindowGtkComponents> = Mutex::new(build_from_glade());

    static ref ADD_OR_UNDO_PURCHASE : Mutex<(Sender<Purchase>, Sender<Purchase>)> = Mutex::new(build_purchase_debouncer());

}

fn redraw_users() {

    let searchterm: &str = {
        let userwindow = GLOBAL_USERWINDOW.lock().unwrap();
        &userwindow.search_bar.get_buffer().get_text()
    };


    //if empty input, show saufbubbies and no action btns
    //if not, show searched subset of users and subset of action btns

    render_user_buttons(searchterm);


}

fn build_purchase_debouncer() -> (Sender<Purchase>, Sender<Purchase>) {
    let (add_tx, add_rx): (Sender<Purchase>, Receiver<Purchase>) = channel();

    let (undo_tx, undo_rx): (Sender<Purchase>, Receiver<Purchase>) = channel();

    {
        thread::spawn(move || {

            let added_purchases = add_rx;
            let undone_purchases = undo_rx;

            //listen to adds, listen to undoes, process results, and sleep for the next cycle

            let mut queue_of_purchases: std::vec::Vec<Purchase> = vec!();

            loop {
                //println!("debouncer loop begins");

                for add in added_purchases.try_iter() {
                    match add {
                        Purchase::SimplePurchase {
                            timestamp_seconds,
                            item_id,
                            consumer_id,
                        } => {
                            queue_of_purchases.push(Purchase::SimplePurchase {
                                timestamp_seconds,
                                item_id,
                                consumer_id,
                            });
                        },
                    }
                }

                for undo in undone_purchases.try_iter() {
                    match undo {
                        Purchase::SimplePurchase {
                            timestamp_seconds,
                            item_id,
                            consumer_id,
                        } => {
                            let ts: u32 = timestamp_seconds;
                            let iid: u32 = item_id;
                            let cid: u32 = consumer_id;

                            queue_of_purchases.retain(|element| match element {
                                &Purchase::SimplePurchase {
                                    ref timestamp_seconds,
                                    ref item_id,
                                    ref consumer_id,
                                } => timestamp_seconds != &ts || item_id != &iid || consumer_id != &cid,
                                _ => true,
                            });
                        },
                    }
                }


                let timestamp: u32 = time::get_time().sec as u32;;


                queue_of_purchases.retain(|element: &Purchase| match element {
                    &Purchase::SimplePurchase {
                        ref timestamp_seconds,
                        ref item_id,
                        ref consumer_id,
                    } => {
                        if (*timestamp_seconds >= timestamp) {
                             return true;
                        } else {
                            println!("Purchase Debounce finished: user {:?}, item {:?}, timestamp {:?}", consumer_id, item_id, timestamp_seconds);
                            finalize_purchase(*consumer_id, *item_id, *timestamp_seconds);
                            return false;
                        }
                    },
                    _ => true,
                });



                thread::sleep(std::time::Duration::from_millis(100)); //TODO: set to something like 10 seconds to deal with notification
            }




            println!("thread finished");
        });
    }


    return (add_tx, undo_tx);

}


fn enqueue_purchase(user_id: u32, item_id: u32, epoch_seconds: u32) {
    //move purchase to
    ADD_OR_UNDO_PURCHASE.lock().unwrap().0.send(Purchase::SimplePurchase {
        consumer_id: user_id,
        item_id: item_id,
        timestamp_seconds: epoch_seconds,
    });
}

fn finalize_purchase(user_id: u32, item_id: u32, epoch_seconds: u32) {
    //set on_idle task to call bl and write to database, followed by all the other interactions

    {
        let exec = move || {

            println!("exec started");

            let bl: &mut RustixBackend<
                TransientPersister,
            > = &mut GLOBAL_BACKEND.lock().expect(
                "Beerlist variable was not available anymore",
            );


            let result = bl.purchase(user_id, item_id, epoch_seconds);
            let item_lbl = &bl.datastore.items[&item_id].name;
            let user_lbl = &bl.datastore.users[&user_id].username;
            render_last_purchase(user_lbl, item_lbl);

            println!("render_last_purchase happened");

            gtk::Continue(false)
        };

        // executes the closure on next chance:
        glib::source::idle_add(exec);

        println!("exec in gtk queue");
    }

}




fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);


    {
        //prepare transient backup a little bit
        let bl: &mut RustixBackend<TransientPersister> = &mut GLOBAL_BACKEND.lock().unwrap();

        bl.create_user("Gruin".to_string());
        bl.create_user("Vall".to_string());
        bl.create_user("rad(i)".to_string());

        for i in 0..99 {
            bl.create_user("GenUser #".to_string() + &i.to_string());
        }

        bl.create_item(
            "Club Mate".to_string(),
            100,
            Some("without alcohol".to_string()),
        );
        bl.create_item("Pils".to_string(), 95, Some("Beer".to_string()));
        bl.create_item("Whiskey".to_string(), 1200, Some("Liquor".to_string()));
        bl.create_item("Schirker".to_string(), 1100, Some("Liquor".to_string()));
        bl.create_item("Kräussen".to_string(), 1100, Some("Beer".to_string()));


        let seed: &[_] = &[42];
        let mut rng: StdRng = SeedableRng::from_seed(seed);


        let mut timestamp_counter = 12345678u32;
        bl.purchase(0, 2, timestamp_counter);

        //random purchases for the existing users
        for user_id in 0..(bl.datastore.users.len() as u32) {
            let nr_of_purchases: u32 = rng.gen_range(0u32, 10u32);
            for _ in 0..nr_of_purchases {
                timestamp_counter += 1;
                let item_id: u32 = rng.gen_range(0u32, bl.datastore.items.len() as u32);
                bl.purchase(user_id, item_id, timestamp_counter);
            }
        }
    }


    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        panic!("GTK failed to initialize")
    }


    window.set_title("Cervisia 6.0");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    window.connect_delete_event(|_, _| {
        //gtk::main_quit();
        Inhibit(false)
    });


    // Creating a vertical layout to place both tree view and label in the window.
    let vertical_layout = gtk::Box::new(Orientation::Vertical, 0);

    // Creation of the label.
    let label = Label::new(None);

    let tree = create_and_setup_view();


    // Setting the model into the view.
    // Adding the view to the layout.

    let scroll = {
        let hadjustment = None;
        let vadjustment = None;
        let wdw = ScrolledWindow::new(hadjustment, vadjustment);
        wdw
    };




    // Adding the layout to the window.
    window.add(&vertical_layout);


    add_application_actions(application, &window);

    //window.show_all();
}

fn render_last_purchase(user: &str, drink: &str) {
    //should be the same as used in the purchase struct
    let timelabel = Local::now().format("%Y-%m-%d %H:%M:%S");

    GLOBAL_USERWINDOW.lock()
                     .expect("Global UserWindow variable does not exist anymore")
                     .log_btn
                     .set_label(&format!(
        "User {} bought 1 {} at {}",
        user,
        drink,
        timelabel
    ));
}

fn render_user_buttons(searchterm: &str) {
    let userwindow: &mut UserWindowGtkComponents = &mut GLOBAL_USERWINDOW.lock().expect(
        "Global UserWindow variable does not exist anymore",
    );

    //take n = 40 top users
    //TODO: check searchterm if non-empty and take 40 users matching the term from all users

    let mut top_users: Vec<u32> = Vec::new();

    if (searchterm.is_empty()) {
        userwindow.action_bar.set_visible(false);



        {
            let bl = GLOBAL_BACKEND.lock().unwrap();
            for element in &bl.datastore.top_users {
                top_users.push(*element);
            }
        }


    } else {
        userwindow.action_bar.set_visible(true);

        //match via suffix tree and take at most NUMBER_OF_USERS_PER_PAGE

        {
            let bl = GLOBAL_BACKEND.lock().unwrap();

            for element in bl.datastore.users_suffix_tree.search(searchterm).iter().take(NUMBER_OF_USERS_PER_PAGE as usize) {
                top_users.push(element.id);
            }
        }

    }



    //println!("Before method: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));





    {
        // println!("Before loop: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));


        for i in 0..NUMBER_OF_USERS_PER_PAGE as usize {
            if i < top_users.len() {
                let user_id: u32 = top_users[i];

                //      println!("Line {}", i);

                {
                    //        println!("{} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));
                    let bl2: &RustixBackend<TransientPersister> = &GLOBAL_BACKEND.lock().unwrap();
                    //set user name as button label
                    userwindow.user_btn[i].set_label(&bl2.datastore.users[&user_id].username);
                }

                //        println!("Middle of loop body: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));

                {
                    //TODO: move this instead to a global variable for user_id, and a global function
                    userwindow.user_btn[i].connect_clicked(move |_| {
                        //              println!("Pressed User ID {}", user_id);
                        let qm = &mut GLOBAL_QUICKMENU.lock().unwrap();
                        let bl = &GLOBAL_BACKEND.lock().unwrap();
                        show_quickmenu(qm, user_id, bl);
                    });
                }


                //            println!("End of loop body: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));

                userwindow.user_btn[i].set_visible(true);
            } else {
                //if top users < 40, set buttons to invisible
                userwindow.user_btn[i].set_visible(false);
            }
        }
    }

    //TODO: deal with action bar, etc.

    //set connect_clicked to call show_quickmenu with user id
}




fn show_quickmenu(
    quickmenu: &mut QuickmenuGtkComponents,
    user_id: u32,
    backend: &rustix_backend::RustixBackend<persistencer::TransientPersister>,
) {
    //TODO: parameters like item strings, item id, and both in 4 options total
    //TODO: user id and user name

    {
        println!("Whole Backend on show_quickmenu = {:?}", backend);

        let drinks_set: &HashSet<u32> = &backend.datastore.top_drinks_per_user[&user_id];

        let mut drinks: Vec<u32> = Vec::new();

        drinks.extend(drinks_set.into_iter());


        quickmenu.quickmenu.show_all();


        for idx in 0..4 {
            {
                println!("drinks length = {}", drinks.len());
            }

            if drinks.len() > idx {
                {
                    let item_id: u32 = drinks[idx];
                    quickmenu.item_btn[idx].set_label(&backend.datastore.items[&item_id].name);
                }
                {
                    let item_id: u32 = drinks[idx];


                    quickmenu.item_btn[idx].connect_clicked(move |_| {
                        let quickmenu = GLOBAL_QUICKMENU.lock()
                                        .expect("Global Window no longer available");

                        quickmenu.close_btn.clicked();


                        let epoch_seconds = time::get_time().sec as u32 + 0; //TODO implement delay here in seconds
                        {

                            println!(
                                "buying {} in quickmenu at epoch seconds {}",
                                idx,
                                epoch_seconds
                            );
                            enqueue_purchase(user_id, item_id, epoch_seconds);


                        }
                    });
                }
                {
                    quickmenu.item_btn[idx].set_visible(true);
                }
            } else {
                quickmenu.item_btn[idx].set_visible(false);
            }
        }
    }
}



fn main() {



    let application = gtk::Application::new("cervisia.gtk", gio::ApplicationFlags::empty()).expect(
        "Initialization failed...",
    );


    {
        let app2 = application.clone();

        application.connect_startup(move |app| {
            build_ui(app);

            let searchterm = "";
            //println!("Before method call: {} weak references and {} strong ones", Rc::weak_count(&quickmenu), Rc::strong_count(&quickmenu));
            render_user_buttons(&searchterm);

            //DELETE THIS: show_quickmenu(&mut quickmenu, 0, backend);


            let result_of_registration = app2.register(None).expect("Registration failed");


        });
    }

    {
        let app2 = application.clone();

        application.connect_activate(move |_| {
            {
                let notification_1 = gio::Notification::new("my notification title 1");

                notification_1.set_body("my notification body with some content");


                notification_1.add_button("My Button", "app.id-notification-undo");

                println!("Sending Notification");

                app2.send_notification("my-notification-id-1", &notification_1);

                println!("Sent Notification");
            }
            {
                let notification_1 = gio::Notification::new("my notification title 2");

                notification_1.set_body("my notification body with some content");


                notification_1.add_button("My Button", "app.id-notification-undo");

                println!("Sending Notification");

                app2.send_notification("my-notification-id-2", &notification_1);

                println!("Sent Notification");
            }
            {
                let notification_1 = gio::Notification::new("my notification title 3");

                notification_1.set_body("my notification body with some content");


                notification_1.add_button("My Button", "app.id-notification-undo");

                println!("Sending Notification");

                app2.send_notification("my-notification-id-3", &notification_1);

                println!("Sent Notification");
            }
        });
    }


    let a: &[&str] = &[];

    std::process::exit(application.run(a));
}


fn create_and_setup_view() -> TreeView {
    // Creating the tree view.
    let tree = TreeView::new();

    tree.set_headers_visible(false);
    // Creating the two columns inside the view.
    append_column(&tree, 0);
    append_column(&tree, 1);
    tree
}


fn append_column(tree: &TreeView, id: i32) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);
    tree.append_column(&column);
}

fn create_and_fill_model(datastore: &Datastore) -> ListStore {
    // Creation of a model with two rows.
    let model = ListStore::new(&[u32::static_type(), String::static_type()]);

    // Filling up the tree view.
    for (_, (id, user)) in datastore.users.iter().enumerate() {
        model.insert_with_values(None, &[0, 1], &[&id, &user.username]);
    }
    model
}


fn add_application_actions(application: &gtk::Application, window: &gtk::ApplicationWindow) {
    let id_notification_undo = gio::SimpleAction::new(
        "id-notification-undo",
        /*Some(glib::VariantTy::new("int"))*/ None,
    );
    id_notification_undo.connect_activate(clone!(window => move |a, b| {
        println!("Something something");
        println!("Received Action with a = {:?} and b = {:?}", a, b);
    }));
    id_notification_undo.set_enabled(true);

    application.add_action(&id_notification_undo);
}



//TODO: show notification whenever a purchase is made, with all data and the undo action
//TODO: build second Thread with 2 channels (one per direction), takes Purchases with date in future and sends a batch to GTK thread whenever 1 or more are "finished"
//TODO: handler function on main thread takes such a Purchase object, and sends it to database AND rewrites current last purchase log with that data
//TODO: need function to translate u32 into string (or chrono struct)
//TODO: the channel to that secondary thread can also take "undo" events. Those will remove a purchase (if possible) before it is sent (if the undo comes later, write an error message to log but ignore undo). Those undo events are spawned by the undo action.




//TODO: create simple function for dealing with password checks (creates a dialog after taking one closure to execute in success). The success function will obviously be executed on the GTK thread, but with the spawnage the whole function becomes async (to wait on user input)
//TODO: don't forget to clear the Password Dialog when showing it

//TODO: whenever top users change (return value of make_purchase), redraw top users and rerender. The same has to be done when the users are changed, example via edit, create or delete



//TODO: add suffix tree (first: mock / quadratic solution). Has to be rebuild during startup AND whenever a user is changed, created or deleted.
//TODO: suffix tree should allow case insensitive searchc (compile flag!)


//TODO: do not show admin actions when searchbar is empty and not in focus. Once it's non-empty, search the available actions. One button can have more than one searchterms (conf that in a file)

//TODO: everytime the focus of the searchbar changes OR the content of it changes, call the rerender function with the correct searchterm. Also show / do not show action buttons as applicable


//TODO: should only care at one line of code if file-based persistence is used or only transient memory


//TODO: reorder / refactor code as following:
/*
Following architecture:
- all dialogs and windows are parsed via lazy_static into a mutex (if performance is low, go with Rc<Refcell> instead
    -  things like the clock are also spawned in those builder functions
- backend is created with via lazy_static too. In the same process the reload-method of the backend is called (before the GUI is created, that's important)
    - for the time being, this is replaced by just loading the mock data
- the main only registers application actions and shows the main window immediately afterwards, after calling an init method
- this init method only calls the right render methods, for example to fill the GUI with the first batch of users
- builder functions and statics are moved to their own file / module
- helper methods are moved to their own file / module
- functions like "show_purchase_notifications" also become their own functions, but get what they need from static variables themselves
*/