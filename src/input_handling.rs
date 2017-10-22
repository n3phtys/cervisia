#![allow(unused_imports)]


use static_variables::GLOBAL_BACKEND;
use static_variables::GLOBAL_QUICKMENU;
use static_variables::GLOBAL_USERWINDOW;
use static_variables::ADD_OR_UNDO_PURCHASE;
use static_variables::USERS_ON_SCREEN;
use static_variables::USER_SELECTED;
use static_variables::ITEMS_ON_SCREEN;
use show_quickmenu;
use enqueue_purchase;
use time;
use suffix::KDTree;
use glade_builders::UserWindowGtkComponents;
use glade_builders::NUMBER_OF_USERS_PER_PAGE;
use blrustix::rustix_backend::RustixBackend;
use blrustix::persistencer::TransientPersister;

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


use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;

use gtk::{Builder, Button, MessageDialog};


pub fn quickmenu_item_btn_pressed(index: usize) {
    let quickmenu = GLOBAL_QUICKMENU.lock()
        .expect("Global Window no longer available");

    quickmenu.close_btn.clicked();


    let epoch_seconds = time::get_time().sec as u32 + 0; //TODO implement delay here in seconds
    {

        println!(
            "buying {} in quickmenu at epoch seconds {}",
            index,
            epoch_seconds
        );


        println!("User_id from last time was {:?}", *USER_SELECTED.lock().unwrap());

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








pub fn render_last_purchase(user: &str, drink: &str) {
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

                {

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

}