#![allow(unused_imports)]


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
use blrustix::datastore::Purchase::SimplePurchase;
use blrustix::persistencer::*;
use blrustix::rustix_backend;
use blrustix::rustix_backend::*;
use cervisia_utilities::*;
use glade_builders::*;
use gtk::Adjustment;
use gtk::ScrolledWindow;
use input_handling::*;
use rand::{Rng, SeedableRng, StdRng};
use static_variables::*;
use std::borrow::*;
use std::cell::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::*;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc::channel;
use std::thread;
use suffix::KDTree;


use chrono::prelude::*;

use static_variables::ADD_OR_UNDO_PURCHASE;
use static_variables::GLOBAL_BACKEND;
use static_variables::GLOBAL_USERWINDOW;
use static_variables::USERS_ON_SCREEN;
use static_variables::USER_SELECTED;

// make moving clones into closures more convenient
/*macro_rules! clone {
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
}*/





fn setup_mock_data() {
    {
        //prepare transient backup a little bit
        let bl: &mut RustixBackend<TransientPersister> = &mut GLOBAL_BACKEND.lock().unwrap();

        bl.create_user("Gruin".to_string());
        bl.create_user("Vall".to_string());
        bl.create_user("rad(i)".to_string());

        for i in 0..250 {
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


        let mut timestamp_counter = 12345678i64;
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


        let target_list: &mut Vec<u32> = &mut ITEMS_ON_SCREEN.lock().unwrap();
        target_list.clear();


        let mut drinks: Vec<u32> = Vec::new();

        drinks.extend(drinks_set.into_iter());


        quickmenu.quickmenu
                 .set_transient_for(&GLOBAL_USERWINDOW.lock().unwrap().application_window);

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
                    target_list.push(item_id);
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
        application.connect_startup(move |app| {
            if gtk::init().is_err() {
                println!("Failed to initialize GTK.");
                panic!("GTK failed to initialize")
            }

            setup_mock_data();

            input_handling::search_entry_text_changed();




            let window: &gtk::ApplicationWindow =
                &GLOBAL_USERWINDOW.lock().unwrap().application_window;
            window.set_application(Some(app));
            window.show();
            add_application_actions(app);


            let _ = app.register(None).expect("Registration failed");
        });
    }

    {
        application.connect_activate(move |_| {});
    }



    let a: &[&str] = &[];


    application.run(a);
}


pub fn show_notification(title: &str, body: &str, id: u32) {
    let application = &GLOBAL_USERWINDOW.lock()
                                        .unwrap()
                                        .application_window
                                        .get_application()
                                        .expect("Application not set!");

    let notification_1 = gio::Notification::new(title);

    notification_1.set_body(body);


    notification_1.add_button("Undo", "app.id-notification-undo");

    println!("Sending Notification");


    let id_lbl = format!("my-notification-id-{}", id);
    application.send_notification(Some(&*id_lbl), &notification_1);

    println!("Sent Notification");
}



fn add_application_actions(application: &gtk::Application) {
    let id_notification_undo = gio::SimpleAction::new(
        "id-notification-undo",
        /*Some(glib::VariantTy::new("int"))*/ None,
    );
    id_notification_undo.connect_activate(move |a, b| {
        println!("Something something");
        println!("Received Action with a = {:?} and b = {:?}", a, b);
    });
    id_notification_undo.set_enabled(true);

    application.add_action(&id_notification_undo);
}



//TODO: show notification whenever a purchase is made, with all data and the undo action
//TODO: build second Thread with 2 channels (one per direction), takes Purchases
// with date in future and sends a batch to GTK thread whenever 1 or more are "finished"
//TODO: handler function on main thread takes such a Purchase object, and sends
// it to database AND rewrites current last purchase log with that data
//TODO: need function to translate u32 into string (or chrono struct)
//TODO: the channel to that secondary thread can also take "undo" events. Those
// will remove a purchase (if possible) before it is sent (if the undo comes later,
// write an error message to log but ignore undo). Those undo events are spawned by
// the undo action.




//TODO: create simple function for dealing with password checks (creates a dialog
// after taking one closure to execute in success). The success function will
// obviously be executed on the GTK thread, but with the spawnage the whole
// function becomes async (to wait on user input)
//TODO: don't forget to clear the Password Dialog when showing it

//TODO: whenever top users change (return value of make_purchase), redraw top users
// and rerender. The same has to be done when the users are changed, example via
// edit, create or delete



//TODO: add suffix tree (first: mock / quadratic solution). Has to be rebuild during
// startup AND whenever a user is changed, created or deleted.
//TODO: suffix tree should allow case insensitive searchc (compile flag!)


//TODO: do not show admin actions when searchbar is empty and not in focus. Once it's
// non-empty, search the available actions. One button can have more than one
// searchterms (conf that in a file)

//TODO: everytime the focus of the searchbar changes OR the content of it changes,
// call the rerender function with the correct searchterm. Also show / do not
// show action buttons as applicable


//TODO: should only care at one line of code if file-based persistence is used or
// only transient memory


//TODO: reorder / refactor code as following:
/*
Following architecture:
- all dialogs and windows are parsed via lazy_static into a mutex (if performance
is low, go with Rc<Refcell> instead
    -  things like the clock are also spawned in those builder functions
- backend is created with via lazy_static too. In the same process the reload-method
of the backend is called (before the GUI is created, that's important)
    - for the time being, this is replaced by just loading the mock data
- the main only registers application actions and shows the main window immediately
afterwards, after calling an init method
- this init method only calls the right render methods, for example to fill the GUI
with the first batch of users
- builder functions and statics are moved to their own file / module
- helper methods are moved to their own file / module
- functions like "show_purchase_notifications" also become their own functions, but
get what they need from static variables themselves
*/
