#![allow(unused_imports)]

use glade_builders::NUMBER_OF_USERS_PER_PAGE;
use glade_builders::UserWindowGtkComponents;
use input_handling::render_last_purchase;
use rustix_bl::persistencer::TransientPersister;
use rustix_bl::rustix_backend::RustixBackend;
use show_quickmenu;
use static_variables::GLOBAL_BACKEND;
use static_variables::GLOBAL_QUICKMENU;
use static_variables::GLOBAL_USERWINDOW;
use static_variables::ITEMS_ON_SCREEN;
use static_variables::USERS_ON_SCREEN;
use static_variables::USER_SELECTED;
use std;
use suffix_rs::KDTree;
use time;

use glib;
use gtk;
use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;

use rustix_bl::*;
use rustix_bl::datastore::*;
use rustix_bl::datastore::Purchase::SimplePurchase;
use rustix_bl::persistencer::*;
use rustix_bl::rustix_backend;
use rustix_bl::rustix_backend::*;

use chrono::*;
use chrono::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::*;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc::channel;
use std::thread;

pub fn current_time() -> String {
    return format!("{} Uhr", Local::now().format("%Y-%m-%d %H:%M:%S"));
}

pub fn enqueue_purchase(user_id: u32, item_id: u32, epoch_millis: i64) {
    finalize_purchase(user_id, item_id, epoch_millis);
}

pub fn finalize_purchase(user_id: u32, item_id: u32, epoch_millis: i64) {
    //set on_idle task to call bl and write to database, followed by all the other interactions

    {
        {
            println!("exec started");

            let bl: &mut RustixBackend<TransientPersister> = &mut GLOBAL_BACKEND.lock().expect(
                "Beerlist variable was not available anymore",
            );


            let _ = bl.purchase(user_id, item_id, epoch_millis); //TODO: use result
            let item_lbl = &bl.datastore.items[&item_id].name;
            let user_lbl = &bl.datastore.users[&user_id].username;
            render_last_purchase(user_lbl, item_lbl);

            println!("render_last_purchase happened");
        }
    }
}
