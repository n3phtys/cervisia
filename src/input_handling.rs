#![allow(unused_imports)]


use enqueue_purchase;
use glade_builders::NUMBER_OF_USERS_PER_PAGE;
use glade_builders::UserWindowGtkComponents;
use gtk::TreeIter;
use gtk::idle_add;
use rustix_bl::datastore::*;
use rustix_bl::persistencer::TransientPersister;
use rustix_bl::rustix_backend::RustixBackend;
use rustix_bl::rustix_backend::WriteBackend;
use show_quickmenu;
use static_variables::GLOBAL_BACKEND;
use static_variables::GLOBAL_QUICKMENU;
use static_variables::GLOBAL_USERWINDOW;
use static_variables::ITEMS_ON_SCREEN;
use static_variables::PURCHASE_SELECTED;
use static_variables::SIZE_OF_PURCHASE_LOG;
use static_variables::USERS_ON_SCREEN;
use static_variables::USER_SELECTED;
use suffix_rs::KDTree;
use time;

use rustix_bl::datastore::Purchaseable;

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


use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;

use gtk::{Builder, Button, MessageDialog};


use show_notification;

pub fn quickmenu_item_btn_pressed(index: usize) {
    {
        show_notification("My title", "My body", 42);
    }

    let quickmenu = GLOBAL_QUICKMENU.lock()
                                    .expect("Global Window no longer available");

    quickmenu.close_btn.clicked();





    let epoch_seconds: i64 = Local::now().timestamp();
    {
        println!(
            "buying {} in quickmenu at epoch seconds {}",
            index,
            epoch_seconds
        );


        println!(
            "User_id from last time was {:?}",
            *USER_SELECTED.lock().unwrap()
        );

        if let Some(ref user_id) = *USER_SELECTED.lock().unwrap() {
            let item_id: u32 = ITEMS_ON_SCREEN.lock().unwrap()[index];


            enqueue_purchase(*user_id, item_id, epoch_seconds);
        }
    }
}

pub fn user_btn_pressed(index: usize) {
    let user_id: u32 = USERS_ON_SCREEN.lock().unwrap()[index];

    let selected: &mut Option<u32> = &mut *USER_SELECTED.lock().unwrap();

    *selected = Some(user_id);


    let qm = &mut GLOBAL_QUICKMENU.lock().unwrap();
    let bl = &GLOBAL_BACKEND.lock().unwrap();
    show_quickmenu(qm, user_id, bl);
}

pub fn search_entry_text_changed() {
    let searchterm: &str = {
        let userwindow = GLOBAL_USERWINDOW.lock().unwrap();
        &userwindow.search_bar.get_buffer().get_text()
    };


    //if empty input, show saufbubbies and no action btns
    //if not, show searched subset of users and subset of action btns

    render_user_buttons(searchterm);
}


pub fn purchase_undo_handler() {
    let opt: &Option<u64> = &*PURCHASE_SELECTED.lock().unwrap();
    if opt.is_some() {
        let id = opt.clone().unwrap();

        let bl = &mut GLOBAL_BACKEND.lock().unwrap();

        let pur: Purchase = bl.datastore.get_purchase(id).unwrap();

        //TODO: implement password check in middle time interval


        //after undone, reselect in purchase log and rerender purchase label

        //TODO: potentially broken? unclear
        let was_the_last = bl.undo_purchase(id);

        //remove entry from purchase log


        let model: &mut ListStore = &mut GLOBAL_USERWINDOW.lock().unwrap().purchase_liststore;
        let size = model.iter_n_children(None);

        let iter: TreeIter = model.get_iter_first().unwrap();




        for n in 0..size {
            println!("iteration with n = {}", n);
            let x: u64 = model.get_value(&iter, 3).get::<u64>().unwrap();
            if x != id {
                model.iter_next(&iter);
            } else {
                //found right iter
                //get previous one if one exists

                println!("iter_nth_child with n = {}", n - 1);
            }
        }


        println!("Reaching this point too");


        //delete element
        model.remove(&iter); //still broken when deleting non-single last


        println!("Reaching this last point");



        //copy new last item from purchase log into purchase label as string (if it was the last)
        println!("Beginning Refresh");
        idle_add(refresh_purchase_label_from_newest_log_element);
        println!("Finished Refresh");
    }
}

pub fn refresh_purchase_label_from_newest_log_element() -> Continue {
    //reselect new last item from purchase log

    let uw = &mut GLOBAL_USERWINDOW.lock().unwrap();

    let log_btn: &Button = &uw.log_btn;

    let model: &ListStore = &uw.purchase_liststore;

    let size = model.iter_n_children(None);
    println!("iter_nth_child with size = {}", size - 1);
    let last_opt = if size - 1 >= 0 {
        model.iter_nth_child(None, size - 1)
    } else {
        None
    };


    log_btn.set_label(
        &&(if last_opt.is_some() {
            let last = last_opt.unwrap();

            let user = model.get_value(&last, 1).get::<String>().unwrap();
            let drink = model.get_value(&last, 2).get::<String>().unwrap();
            let secs = model.get_value(&last, 0).get::<i64>().unwrap();
            let timelabel = Local.timestamp(secs, 0).format("%Y-%m-%d %H:%M:%S");

            let x = format!("User {} bought 1 {} at {}", user, drink, &timelabel).to_string();

            x
        } else {
            "No purchases since last bill".to_string()
        }),
    );

    Continue(false)
}

pub fn handle_purchase_unselect() -> Continue {
    {
        let selected: &mut Option<u64> = &mut *PURCHASE_SELECTED.lock().unwrap();
        *selected = None;
    }

    let w = &mut GLOBAL_USERWINDOW.lock()
                                  .expect("Global UserWindow variable does not exist anymore");

    w.undo_purchase_btn.set_sensitive(false);
    w.log_upper_label.set_text("");
    w.log_lower_label.set_text("");


    Continue(false)
}

pub fn handle_purchase_select(id: u64) {
    println!("handle_purchase_select A");

    let selected: &mut Option<u64> = &mut *PURCHASE_SELECTED.lock().unwrap();
    *selected = Some(id);


    println!("handle_purchase_select B");


    let bl = GLOBAL_BACKEND.lock().unwrap();


    println!("handle_purchase_select C for id = {}", id);

    match bl.datastore.get_purchase(id) {
        //TODO: here comes out nonsense, with 0-2 as ids for consumer and item
        Some(Purchase::SimplePurchase {
            unique_id,
            timestamp_epoch_millis,
            item_id,
            consumer_id,
        }) => {
            println!(
                "handle_purchase_select D for unique_id = {}, timestamp = {},\
                 item_id = {}, consumer_id = {}",
                unique_id,
                timestamp_epoch_millis,
                item_id,
                consumer_id
            );


            let w = &mut GLOBAL_USERWINDOW.lock().expect(
                "Global UserWindow variable does not exist anymore",
            );


            println!("handle_purchase_select E");

            w.undo_purchase_btn.set_sensitive(true);
            println!(
                "username = {} for id {}",
                &bl.datastore.users[&consumer_id].username,
                consumer_id
            );
            w.log_upper_label.set_text(&bl.datastore.users[&consumer_id].username);
            println!(
                "itemname = {} for id {}",
                &bl.datastore.items[&item_id].name,
                item_id
            );
            w.log_lower_label.set_text(&bl.datastore.items[&item_id].name);



            println!("handle_purchase_select F");
        }
        _ => {}
    }

    println!("handle_purchase_select G");
}


pub fn render_last_purchase(user: &str, drink: &str, ts: i64, id: u64) {
    //should be the same as used in the purchase struct
    let timestamp = Local::now();
    let timelabel = timestamp.format("%Y-%m-%d %H:%M:%S");

    {
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

    {
        //add to purchase log list model (and potentially remove an old one)
        let model: &mut ListStore = &mut GLOBAL_USERWINDOW.lock().unwrap().purchase_liststore;

        let user_lbl: &&&str = &&user;
        let item_lbl: &&&str = &&drink;
        let epochmillis: &i64 = &ts;
        let id: &u64 = &id;

        model.insert_with_values(None, &[0, 1, 2, 3], &[epochmillis, item_lbl, user_lbl, id]);

        //remove oldest one from model if n > 200, that is position = 0
        let position0opt = model.get_iter_first();
        //get size
        let size = model.iter_n_children(None);

        println!("Found {} elements in model", size);

        match position0opt {
            Some(treeiter) => if size > SIZE_OF_PURCHASE_LOG as i32 {
                model.remove(&treeiter);
            },
            None => {
                panic!("No first position element found");
            }
        }
    }
}





fn render_user_buttons(searchterm: &str) {
    let userwindow: &mut UserWindowGtkComponents = &mut GLOBAL_USERWINDOW.lock().expect(
        "Global UserWindow variable does not exist anymore",
    );

    //take n = 40 top users
    //check searchterm if non-empty and take 40 users matching the term from all users

    let mut top_users: Vec<u32> = Vec::new();

    if searchterm.is_empty() {
        userwindow.action_bar.set_visible(false);



        let target_list: &mut Vec<u32> = &mut USERS_ON_SCREEN.lock().unwrap();

        target_list.clear();

        {
            let bl = GLOBAL_BACKEND.lock().unwrap();
            for element in &bl.datastore.top_users {
                top_users.push(*element);
                target_list.push(*element);
            }
        }
    } else {
        userwindow.action_bar.set_visible(true);

        //match via suffix tree and take at most NUMBER_OF_USERS_PER_PAGE

        {
            let bl = GLOBAL_BACKEND.lock().unwrap();

            let suff = bl.datastore.users_suffix_tree.search(searchterm);

            let list = suff.iter().take(NUMBER_OF_USERS_PER_PAGE as usize);

            let target_list: &mut Vec<u32> = &mut USERS_ON_SCREEN.lock().unwrap();

            target_list.clear();

            for element in list {
                top_users.push(element.id);
                target_list.push(element.id);
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

                {}


                //            println!("End of loop body: {} weak references and {} strong ones", Rc::weak_count(&backend), Rc::strong_count(&backend));

                userwindow.user_btn[i].set_visible(true);
            } else {
                //if top users < 40, set buttons to invisible
                userwindow.user_btn[i].set_visible(false);
            }
        }
    }

    //TODO: deal with action bar, etc.
}
