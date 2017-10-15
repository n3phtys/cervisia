extern crate gio;
extern crate glib;
extern crate gtk;
extern crate rand;

use gtk::prelude::*;
use gtk::{
    CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn,
    Window, WindowPosition, WindowType
};

use gtk::{Builder, Button, MessageDialog};

use gio::prelude::*;
use gtk::{
    AboutDialog, AboutDialogExt, BoxExt, ContainerExt, DialogExt, GtkApplicationExt,
    Inhibit, LabelExt, SwitchExt, ToVariant, WidgetExt
};
use std::env;


#[macro_use]
extern crate closet;
extern crate blrustix;

#[macro_use]
extern crate lazy_static;


use std::ops::DerefMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use blrustix::*;
use blrustix::rustix_backend::*;
use blrustix::persistencer::*;
use blrustix::datastore::*;
use blrustix::rustix_backend;
use std::cell::*;
use std::ops::*;
use std::borrow::*;
use gtk::ScrolledWindow;
use std::collections::HashSet;
use std::collections::HashMap;
use gtk::Adjustment;
use rand::{Rng, SeedableRng, StdRng};


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


//static mut global_x: u32 = 42;


unsafe impl Sync for QuickmenuGtkComponents {} //hack
unsafe impl Send for QuickmenuGtkComponents {} //hack

lazy_static! {
    static ref GLOBAL_BACKEND: Mutex<rustix_backend::RustixBackend<persistencer::TransientPersister>> = Mutex::new(blrustix::build_transient_backend());
    static ref GLOBAL_QUICKMENU: Mutex<QuickmenuGtkComponents> = Mutex::new(build_quickmenu());
}



fn build_ui(application: &gtk::Application) -> Rc<RefCell<rustix_backend::RustixBackend<persistencer::TransientPersister>>> {
    let window = gtk::ApplicationWindow::new(application);




    //window.connect_keys_changed(move |key| {
    //let k: gtk::Key = key;
    //println!("key = {:?}", key);
    //std::process::exit(0);
    //});


    let mut backend = Rc::new(RefCell::new(blrustix::build_transient_backend()));

    {
        //prepare transient backup a little bit
        let mut bl2 = &*Rc::get_mut(&mut backend).unwrap();
        let mut bl3 = bl2.borrow_mut();
        let bl: &mut RustixBackend<TransientPersister> = bl3.deref_mut();

        bl.create_user("Gruin".to_string());
        bl.create_user("Vall".to_string());
        bl.create_user("rad(i)".to_string());

        for i in 0..99 {
            bl.create_user("GenUser #".to_string() + &i.to_string());
        }

        bl.create_item("Club Mate".to_string(), 100, Some("without alcohol".to_string()));
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


    let model = {
        let bl2 = &*Rc::get_mut(&mut backend).unwrap();
        let bl3 = bl2.borrow();
        let bl: &RustixBackend<TransientPersister> = bl3.deref();

        create_and_fill_model(&bl.datastore)
    };

    // Setting the model into the view.
    tree.set_model(Some(&model));
    // Adding the view to the layout.

    let scroll = {
        let hadjustment = None;
        let vadjustment = None;
        let wdw = ScrolledWindow::new(hadjustment, vadjustment);
        wdw
    };

    vertical_layout.add(&scroll);

    {
        scroll.add(&tree);
    }

    // Same goes for the label.
    vertical_layout.add(&label);

    // The closure responds to selection changes by connection to "::cursor-changed" signal,
    // that gets emitted when the cursor moves (focus changes).
    tree.connect_cursor_changed(move |tree_view| {
        let selection = tree_view.get_selection();
        if let Some((model, iter)) = selection.get_selected() {
            // Now getting back the values from the row corresponding to the
            // iterator `iter`.
            //
            // The `get_value` method do the conversion between the gtk type and Rust.
            label.set_text(&format!("Hello '{}' from row {}",
                                    model.get_value(&iter, 1).get::<String>().unwrap(),
                                    model.get_value(&iter, 0).get::<u32>().unwrap()));
        }
    });


    // Adding the layout to the window.
    window.add(&vertical_layout);


    add_application_actions(application, &window);

    //window.show_all();

    return backend;
}

struct UserWindowGtkComponents {
    user_btn: [gtk::Button; 40],
    action_btn: [gtk::Button; 5],
    action_bar: gtk::ButtonBox,
    clock_label: gtk::Label,
    log_btn: gtk::Button,
    search_bar: gtk::Entry,
}


fn render_user_buttons(searchterm: &str, quickmenu: Rc<RefCell<QuickmenuGtkComponents>>, userwindow: &mut UserWindowGtkComponents, mut backend: Rc<RefCell<rustix_backend::RustixBackend<persistencer::TransientPersister>>>) {
    //take n = 40 top users
    //TODO: check searchterm if non-empty and take 40 users matching the term from all users



    println!("Before method: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));


    let mut top_users: Vec<u32> = Vec::new();

    {
        let mut bl2 = &*Rc::get_mut(&mut backend).unwrap();
        let mut bl3 = bl2.borrow_mut();
        let bl: &mut RustixBackend<TransientPersister> = bl3.deref_mut();

        for element in &bl.datastore.top_users {
            top_users.push(*element);
        }
    }



    {



        println!("Before loop: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));


    for i in 0..40 {
        if i < top_users.len() {
            let user_id: u32 = top_users[i];

            println!("Line {}", i);

            {
                println!("{} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));
                let bl2: &RustixBackend<TransientPersister> = &Rc::deref(&backend).borrow();
                //set user name as button label
                userwindow.user_btn[i].set_label(&bl2.datastore.users[&user_id].username);
            }

            println!("Middle of loop body: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));

            {
                let mut backend2 = Rc::downgrade(&backend);
                let mut quickmenu2 = Rc::downgrade(&quickmenu);


                userwindow.user_btn[i].connect_clicked(clone_army!([quickmenu2, backend2, user_id] move |_| {
                println!("Pressed User ID {}", user_id);
                let mut backend3 = backend2.upgrade().expect("Upgrade of Weak Backend failed");
                let mut quickmenu3 = quickmenu2.upgrade().expect("Upgrade of Weak Quickmenu failed");
                let mut qm2 = &*Rc::get_mut(&mut quickmenu3).unwrap();
    let mut qm3 = qm2.borrow_mut();
    let qm: &mut QuickmenuGtkComponents = qm3.deref_mut();
                show_quickmenu(qm, user_id, backend3);
            }));
            }


            println!("End of loop body: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));

            userwindow.user_btn[i].set_visible(true);
        } else {
            //if top users < 40, set buttons to invisible
            userwindow.user_btn[i].set_visible(false);
        }
    }



        let backend = Rc::downgrade(&backend);
        let quickmenu = Rc::downgrade(&quickmenu);

}

    //TODO: deal with action bar, etc.

    //set connect_clicked to call show_quickmenu with user id


}




struct QuickmenuGtkComponents {
    quickmenu: gtk::Dialog,
    item_btn: [gtk::Button; 4],
    other_drinks: gtk::Button,
    free_be: gtk::Button,
    statistics: gtk::Button,
}

fn build_quickmenu() -> QuickmenuGtkComponents {
    let glade_src = include_str!("quickmenu.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::Dialog = builder.get_object("quickmenu").expect("Couldn't get quickmenu");

    let close_btn: gtk::Button = builder.get_object("close_dialog").expect("Couldn't get quickmenu");

    close_btn.connect_clicked(move |_| {
        window.hide();
    });


    return QuickmenuGtkComponents {
        quickmenu: builder.get_object("quickmenu").expect("Couldn't get quickmenu"),
        item_btn: [builder.get_object("item_btn_0").expect("Couldn't get item_btn_0"), builder.get_object("item_btn_1").expect("Couldn't get item_btn_1"), builder.get_object("item_btn_2").expect("Couldn't get item_btn_2"), builder.get_object("item_btn_3").expect("Couldn't get item_btn_3")],
        other_drinks: builder.get_object("andere_getraenke").expect("Couldn't get andere_getraenke"),
        free_be: builder.get_object("ausgeben").expect("Couldn't get ausgeben"),
        statistics: builder.get_object("statistik").expect("Couldn't get statistik"),
    };
}


fn show_quickmenu(quickmenu: &mut QuickmenuGtkComponents, user_id: u32, mut backend: Rc<RefCell<rustix_backend::RustixBackend<persistencer::TransientPersister>>>) {
    //TODO: parameters like item strings, item id, and both in 4 options total
    //TODO: user id and user name

    {
        let mut bl2 = &*Rc::get_mut(&mut backend).unwrap();
        let mut bl3 = bl2.borrow_mut();
        let bl: &mut RustixBackend<TransientPersister> = bl3.deref_mut();

        let drinks_set: &HashSet<u32> = &bl.datastore.top_drinks_per_user[&user_id];

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
                    quickmenu.item_btn[idx].set_label(&bl.datastore.items[&item_id].name);
                }
                {
                    let item_id: u32 = drinks[idx];
                    quickmenu.item_btn[idx].connect_clicked(move |_| {
                        println!("clicked on {}", idx);
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

fn build_from_glade() -> UserWindowGtkComponents {
    let glade_src = include_str!("main-window.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::ApplicationWindow = builder.get_object("user_selection_window").expect("Couldn't get user_selection_window");

    window.show_all();


    let builder2 = Builder::new_from_string(glade_src);

    let get_placeholder = clone_army!([builder] move || {
        let placeholder = builder.get_object("action_btn_0").expect("Couldn't get action_btn_0");
        return placeholder;
    });


    let mut action_btns: [gtk::Button; 5] = [get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder()];
    let mut user_btns: [gtk::Button; 40] = [get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(),get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(),get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(),get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(),get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(),get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(),get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(),get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder(), get_placeholder()];

    for i in 0..5 {
        let id = format!("action_btn_{}", i);
        let errormsg = format!("Couldn't get action_btn_{}", i);
        action_btns[i] = builder.get_object(&id).expect(&errormsg);
    }
    for i in 0..40 {
        let id = format!("user_btn_{}", i);
        let errormsg = format!("Couldn't get user_btn_{}", i);
        user_btns[i] = builder.get_object(&id).expect(&errormsg);
    }

    let action_box_bar : gtk::ButtonBox = builder.get_object("action_bar").expect("Couldn't get action_bar");
    let clock_time_label : gtk::Label = builder.get_object("clock_label").expect("Couldn't get clock_label");
    let purchase_log_btn: gtk::Button = builder.get_object("log_btn").expect("Couldn't get log_btn");
    let search_entry: gtk::Entry = builder.get_object("search_bar").expect("Couldn't get search_bar");

    return UserWindowGtkComponents {
        user_btn: user_btns,
        action_btn: action_btns,
        action_bar: action_box_bar,
        clock_label: clock_time_label,
        log_btn: purchase_log_btn,
        search_bar: search_entry,
    };
}


fn main() {
    let application = gtk::Application::new("cervisia.gtk", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");


    {
        let app2 = application.clone();

        application.connect_startup(move |app| {
            let mut backend = build_ui(app);


            let mut user_window = build_from_glade();
            let mut quickmenu = Rc::new(RefCell::new( build_quickmenu()));
            let searchterm = "";
            println!("Before method call: {} weak references and {} strong ones", Rc::weak_count(&quickmenu), Rc::strong_count(&quickmenu));
            render_user_buttons(&searchterm, quickmenu, &mut user_window, backend);

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
    let id_notification_undo = gio::SimpleAction::new("id-notification-undo", /*Some(glib::VariantTy::new("int"))*/ None);
    id_notification_undo.connect_activate(clone!(window => move |a, b| {
        println!("Something something");
        println!("Received Action with a = {:?} and b = {:?}", a, b);
    }));
    id_notification_undo.set_enabled(true);

    application.add_action(&id_notification_undo);
}