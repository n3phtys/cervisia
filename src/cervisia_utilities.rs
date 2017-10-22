#![allow(unused_imports)]

use static_variables::GLOBAL_BACKEND;
use static_variables::GLOBAL_QUICKMENU;
use static_variables::GLOBAL_USERWINDOW;
use static_variables::ADD_OR_UNDO_PURCHASE;
use static_variables::USERS_ON_SCREEN;
use static_variables::USER_SELECTED;
use static_variables::ITEMS_ON_SCREEN;
use show_quickmenu;
use time;
use suffix::KDTree;
use glade_builders::UserWindowGtkComponents;
use glade_builders::NUMBER_OF_USERS_PER_PAGE;
use blrustix::rustix_backend::RustixBackend;
use blrustix::persistencer::TransientPersister;
use std;
use input_handling::render_last_purchase;

use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;
use gtk;
use glib;

use blrustix::*;
use blrustix::datastore::*;
use blrustix::persistencer::*;
use blrustix::rustix_backend;
use blrustix::rustix_backend::*;
use blrustix::datastore::Purchase::SimplePurchase;

use chrono::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::*;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use chrono::prelude::*;

pub fn current_time() -> String {
    return format!("{} Uhr", Local::now().format("%Y-%m-%d %H:%M:%S"));
}




pub fn build_purchase_debouncer() -> (Sender<Purchase>, Sender<Purchase>) {
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
                        if *timestamp_seconds >= timestamp {
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

        });
    }


    return (add_tx, undo_tx);

}


pub fn enqueue_purchase(user_id: u32, item_id: u32, epoch_seconds: u32) {
    //move purchase to
    ADD_OR_UNDO_PURCHASE.lock().unwrap().0.send(Purchase::SimplePurchase {
        consumer_id: user_id,
        item_id: item_id,
        timestamp_seconds: epoch_seconds,
    });
}

pub fn finalize_purchase(user_id: u32, item_id: u32, epoch_seconds: u32) {
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
