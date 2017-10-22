#![allow(unused_imports)]

use gtk;


use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;

use gtk::{Builder, Button, MessageDialog};

use gio::prelude::*;
use gtk::{AboutDialog, AboutDialogExt, BoxExt, ContainerExt, DialogExt, GtkApplicationExt,
          Inhibit, LabelExt, SwitchExt, ToVariant, WidgetExt};
use std;
use std::env;

use input_handling::quickmenu_item_btn_pressed;
use input_handling::user_btn_pressed;


use cervisia_utilities::current_time;
use input_handling::search_entry_text_changed;

pub const NUMBER_OF_USERS_PER_PAGE: u8 = 40;

pub struct QuickmenuGtkComponents {
    pub quickmenu: gtk::Dialog,
    pub item_btn: [gtk::Button; 4],
    pub other_drinks: gtk::Button,
    pub free_be: gtk::Button,
    pub statistics: gtk::Button,
    pub close_btn: gtk::Button,
}

pub fn build_quickmenu() -> QuickmenuGtkComponents {
    let glade_src = include_str!("quickmenu.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::Dialog = builder.get_object("quickmenu")
                                     .expect("Couldn't get quickmenu");

    let close_btn: gtk::Button = builder.get_object("close_dialog")
                                        .expect("Couldn't get quickmenu");

    close_btn.connect_clicked(move |_| {
        window.hide();
    });

    let item_btns: [gtk::Button; 4 as usize] = [
        builder.get_object("item_btn_0")
               .expect("Couldn't get item_btn_0"),
        builder.get_object("item_btn_1")
               .expect("Couldn't get item_btn_1"),
        builder.get_object("item_btn_2")
               .expect("Couldn't get item_btn_2"),
        builder.get_object("item_btn_3")
               .expect("Couldn't get item_btn_3"),
    ];

    for i in 0..4 {
        let k = i;
        item_btns[i].connect_clicked(move |_| {
            quickmenu_item_btn_pressed(k);
        });
    }


    return QuickmenuGtkComponents {
        quickmenu: builder.get_object("quickmenu")
                          .expect("Couldn't get quickmenu"),
        item_btn: item_btns,
        other_drinks: builder.get_object("andere_getraenke")
                             .expect("Couldn't get andere_getraenke"),
        free_be: builder.get_object("ausgeben")
                        .expect("Couldn't get ausgeben"),
        statistics: builder.get_object("statistik")
                           .expect("Couldn't get statistik"),
        close_btn: close_btn,
    };
}


pub fn build_from_glade() -> UserWindowGtkComponents {
    let glade_src = include_str!("main-window.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::ApplicationWindow = builder.get_object("user_selection_window")
                                                .expect("Couldn't get user_selection_window");

    window.show_all();


    let get_placeholder = clone_army!([builder] move || {
let placeholder = builder.get_object("action_btn_0").expect("Couldn't get action_btn_0");
return placeholder;
});


    let mut action_btns: [gtk::Button; 6] = [
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
    ];
    let mut user_btns: [gtk::Button; NUMBER_OF_USERS_PER_PAGE as usize] = [
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
        get_placeholder(),
    ];

    for i in 0..6 {
        let id = format!("action_btn_{}", i);
        let errormsg = format!("Couldn't get action_btn_{}", i);
        action_btns[i] = builder.get_object(&id).expect(&errormsg);
    }
    //close button
    action_btns[5].connect_clicked(move |_| {
        std::process::exit(0);
    });
    for i in 0..NUMBER_OF_USERS_PER_PAGE as usize {
        let id = format!("user_btn_{}", i);
        let errormsg = format!("Couldn't get user_btn_{}", i);
        user_btns[i] = builder.get_object(&id).expect(&errormsg);

        user_btns[i].connect_clicked(move |_| {
            user_btn_pressed(i);
        });
    }

    let action_box_bar: gtk::ButtonBox = builder.get_object("action_bar")
                                                .expect("Couldn't get action_bar");
    let clock_time_label: gtk::Label = builder.get_object("clock_label")
                                              .expect("Couldn't get clock_label");

    {
        let clock_clone = clock_time_label.clone();
        let time = current_time();
        clock_clone.set_text(&time);

        let tick = move || {
            let time = current_time();
            clock_clone.set_text(&time);
            gtk::Continue(true)
        };

        // executes the closure once every second
        gtk::timeout_add_seconds(1, tick);
    }

    let purchase_log_btn: gtk::Button =
        builder.get_object("log_btn").expect("Couldn't get log_btn");
    let search_entry: gtk::SearchEntry = builder.get_object("search_bar")
                                                .expect("Couldn't get search_bar");

    {
        search_entry.connect_search_changed(move |_| {
            search_entry_text_changed();
        });
    }

    return UserWindowGtkComponents {
        application_window: builder.get_object("user_selection_window")
                                   .expect("Couldn't get user_selection_window"),
        user_btn: user_btns,
        action_btn: action_btns,
        action_bar: action_box_bar,
        clock_label: clock_time_label,
        log_btn: purchase_log_btn,
        search_bar: search_entry,
    };
}


pub struct UserWindowGtkComponents {
    pub application_window: gtk::ApplicationWindow,
    pub user_btn: [gtk::Button; NUMBER_OF_USERS_PER_PAGE as usize],
    pub action_btn: [gtk::Button; 6],
    pub action_bar: gtk::ButtonBox,
    pub clock_label: gtk::Label,
    pub log_btn: gtk::Button,
    pub search_bar: gtk::SearchEntry,
}




unsafe impl Sync for QuickmenuGtkComponents {} //hack
unsafe impl Send for QuickmenuGtkComponents {} //hack
unsafe impl Sync for UserWindowGtkComponents {} //hack
unsafe impl Send for UserWindowGtkComponents {} //hack
