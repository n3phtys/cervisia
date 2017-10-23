#![allow(unused_imports)]

use blrustix::persistencer::TransientPersister;
use blrustix::rustix_backend::RustixBackend;
use glade_builders::NUMBER_OF_USERS_PER_PAGE;
use glade_builders::UserWindowGtkComponents;
use input_handling::render_last_purchase;
use show_quickmenu;
use static_variables::ADD_OR_UNDO_PURCHASE;
use static_variables::GLOBAL_BACKEND;
use static_variables::GLOBAL_QUICKMENU;
use static_variables::GLOBAL_USERWINDOW;
use static_variables::ITEMS_ON_SCREEN;
use static_variables::USERS_ON_SCREEN;
use static_variables::USER_SELECTED;
use std;
use suffix::KDTree;
use time;

use glib;
use gtk;
use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;

use blrustix::*;
use blrustix::datastore::*;
use blrustix::datastore::Purchase::SimplePurchase;
use blrustix::persistencer::*;
use blrustix::rustix_backend;
use blrustix::rustix_backend::*;

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




pub fn build_purchase_debouncer() -> (Sender<Purchase>, Sender<Purchase>) {
    let (add_tx, add_rx): (Sender<Purchase>, Receiver<Purchase>) = channel();

    let (undo_tx, undo_rx): (Sender<Purchase>, Receiver<Purchase>) = channel();

    {
        thread::spawn(move || {
            let added_purchases = add_rx;
            let undone_purchases = undo_rx;

            //listen to adds, listen to undoes, process results, and sleep for the next cycle

            let mut queue_of_purchases: std::vec::Vec<Purchase> = vec![];

            loop {
                //println!("debouncer loop begins");

                for add in added_purchases.try_iter() {
                    match add {
                        Purchase::SimplePurchase {
                            unique_id,
                            timestamp_epoch_millis,
                            item_id,
                            consumer_id,
                        } => {
                            queue_of_purchases.push(Purchase::SimplePurchase {
                                unique_id,
                                timestamp_epoch_millis,
                                item_id,
                                consumer_id,
                            });
                        }
                        Purchase::UndoPurchase { ref unique_id } => unimplemented!(),
                    }
                }

                for undo in undone_purchases.try_iter() {
                    match undo {
                        Purchase::SimplePurchase {
                            unique_id,
                            timestamp_epoch_millis,
                            item_id,
                            consumer_id,
                        } => {
                            let ts: i64 = timestamp_epoch_millis;
                            let iid: u32 = item_id;
                            let cid: u32 = consumer_id;

                            queue_of_purchases.retain(|element| match element {
                                &Purchase::SimplePurchase {
                                    ref unique_id,
                                    ref timestamp_epoch_millis,
                                    ref item_id,
                                    ref consumer_id,
                                } => {
                                    timestamp_epoch_millis != &ts || item_id != &iid
                                        || consumer_id != &cid
                                }
                                &Purchase::UndoPurchase { ref unique_id } => unimplemented!(),
                            });
                        }
                        Purchase::UndoPurchase { ref unique_id } => unimplemented!(),
                    }
                }


                let timestamp = Local::now().timestamp();


                queue_of_purchases.retain(|element: &Purchase| match element {
                    &Purchase::SimplePurchase {
                        ref unique_id,
                        ref timestamp_epoch_millis,
                        ref item_id,
                        ref consumer_id,
                    } => if *timestamp_epoch_millis >= timestamp {
                        return true;
                    } else {
                        println!(
                            "Purchase Debounce finished: user {:?}, item {:?}, timestamp {:?}",
                            consumer_id,
                            item_id,
                            timestamp_epoch_millis
                        );
                        finalize_purchase(*consumer_id, *item_id, *timestamp_epoch_millis);
                        return false;
                    },
                    &Purchase::UndoPurchase { ref unique_id } => {
                        return true;
                    }
                });



                thread::sleep(std::time::Duration::from_millis(100)); //TODO: set to something like 10 seconds to deal with notification
            }
        });
    }


    return (add_tx, undo_tx);
}


pub fn enqueue_purchase(user_id: u32, item_id: u32, epoch_millis: i64) {
    //move purchase to
    let _ = ADD_OR_UNDO_PURCHASE.lock()
                                .unwrap()
                                .0
                                .send(Purchase::SimplePurchase {
        unique_id: 0u64,
        consumer_id: user_id,
        item_id: item_id,
        timestamp_epoch_millis: epoch_millis,
    });
}

pub fn finalize_purchase(user_id: u32, item_id: u32, epoch_millis: i64) {
    //set on_idle task to call bl and write to database, followed by all the other interactions

    {
        let exec = move || {
            println!("exec started");

            let bl: &mut RustixBackend<TransientPersister> = &mut GLOBAL_BACKEND.lock().expect(
                "Beerlist variable was not available anymore",
            );


            let _ = bl.purchase(user_id, item_id, epoch_millis); //TODO: use result
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
