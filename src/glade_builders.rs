#![allow(unused_imports)]

use gtk;


use gtk::{CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn, Window,
          WindowPosition, WindowType};
use gtk::prelude::*;

use serde_json::Value;

use gtk::{Builder, Button, MessageDialog};
use std::sync::Mutex;

use gio::prelude::*;
use gtk::{AboutDialog, AboutDialogExt, BoxExt, ContainerExt, DialogExt, GtkApplicationExt,
          Inhibit, LabelExt, SwitchExt, ToVariant, WidgetExt};
use std;
use std::env;

use input_handling::purchase_undo_handler;
use input_handling::quickmenu_item_btn_pressed;
use input_handling::user_btn_pressed;

use cervisia_utilities::current_time;
use input_handling::handle_purchase_select;
use input_handling::handle_purchase_unselect;
use input_handling::search_entry_text_changed;
use static_variables::PURCHASE_SELECTED;

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


    let close_log: gtk::Button = builder.get_object("close_log_btn")
                                        .expect("Couldn't get close_log_btn");
    let undo_log: gtk::Button = builder.get_object("undo_purchase_btn")
                                       .expect("Couldn't get undo_purchase_btn");
    let log_upper_label: gtk::Label = builder.get_object("log_upper_label")
                                             .expect("Couldn't get log_upper_label");
    let log_lower_label: gtk::Label = builder.get_object("log_lower_label")
                                             .expect("Couldn't get log_lower_label");
    let purchase_log_listview: gtk::TreeView = builder.get_object("purchase_log_listview")
                                                      .expect("Couldn't get purchase_log_listview");
    let purchase_liststore: gtk::ListStore = builder.get_object("purchase_liststore")
                                                    .expect("Couldn't get purchase_liststore");

    let wizard_gen: gtk::Assistant = builder.get_object("wizard_gen").expect("Couldn't get wizard_gen");


    {
        purchase_liststore.clear();
    }

    let purchase_log: gtk::Dialog = builder.get_object("purchase_log")
                                           .expect("Couldn't get purchase_log");


    {
        let purchase_log = purchase_log.clone();
        close_log.connect_clicked(move |_| {
            purchase_log.hide();
        });
    }

    {
        let purchase_log = purchase_log.clone();
        purchase_log_btn.connect_clicked(move |_| {
            purchase_log.show();
        });
    }


    {
        undo_log.connect_clicked(move |_| {
            purchase_undo_handler();
        });
    }

    //connect selection handling with global variable
    {
        purchase_log_listview.connect_cursor_changed(move |tree_view| {
            println!("selection gotten");
            let selection = tree_view.get_selection();
            println!("selection tested");
            if let Some((model, iter)) = selection.get_selected() {
                println!("selection exists");
                let id: u64 = model.get_value(&iter, 3)
                                   .get::<u64>()
                                   .expect("Couldn't get purchase id value");
                idle_add(move || {
                    handle_purchase_select(id);
                    Continue(false)
                });
            } else {
                idle_add(handle_purchase_unselect);
            }
        });
    }

    {
        undo_log.set_sensitive(false);
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
        purchase_log: purchase_log,
        purchase_log_listview: purchase_log_listview,
        log_upper_label: log_upper_label,
        log_lower_label: log_lower_label,
        undo_purchase_btn: undo_log,
        close_log_btn: close_log,
        purchase_liststore: purchase_liststore,
        wizard_gen: wizard_gen,
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
    pub purchase_log: gtk::Dialog,
    pub purchase_log_listview: gtk::TreeView,
    pub log_upper_label: gtk::Label,
    pub log_lower_label: gtk::Label,
    pub undo_purchase_btn: gtk::Button,
    pub close_log_btn: gtk::Button,
    pub purchase_liststore: gtk::ListStore,
    pub wizard_gen: gtk::Assistant,
}




unsafe impl Sync for QuickmenuGtkComponents {} //hack
unsafe impl Send for QuickmenuGtkComponents {} //hack
unsafe impl Sync for UserWindowGtkComponents {} //hack
unsafe impl Send for UserWindowGtkComponents {} //hack



pub trait AssistantDialogueable{

    fn unique_assistant_id(&self) -> &str;

    fn get_multiselect_tree_store(&self) -> gtk::TreeModel;
    fn get_singleselect_tree_store(&self) -> gtk::TreeModel;
    fn get_combo_box_primary_type(&self) -> Vec<(&str, u64)>;
    fn get_autocomplete_words(&self) -> Vec<&str>;

    fn has_multi_select_view(&self) -> bool;
    fn has_single_select_view(&self) -> bool;
    fn has_primary_typecombo_box(&self) -> bool;
    fn has_normal_text_field(&self) -> bool;
    fn has_autocomplete_text_field(&self) -> bool;
    fn has_u64input_field(&self) -> bool;

    fn title(&self) -> &str;
    fn description(&self) -> &str;
    fn apply_label(&self) -> &str;

    fn multi_select_label(&self) -> &str;
    fn single_select_label(&self) -> &str;
    fn combo_box_label(&self) -> &str;
    fn normal_text_field_label(&self) -> &str;
    fn autocomplete_text_field_label(&self) -> &str;
    fn u64input_label(&self) -> &str;

    fn assistant_apply_result_json(&self, json: Value);

}

pub enum AssistantDialogue {
    Donation,
    UserAdministration,
    ItemAdministration
}

impl AssistantDialogueable for AssistantDialogue {
    fn unique_assistant_id(&self) -> &str {
        unimplemented!()
    }

    fn get_multiselect_tree_store(&self) -> gtk::TreeModel {
        unimplemented!()
    }

    fn get_singleselect_tree_store(&self) -> gtk::TreeModel {
        unimplemented!()
    }

    fn get_combo_box_primary_type(&self) -> Vec<(&str, u64)> {
        unimplemented!()
    }

    fn get_autocomplete_words(&self) -> Vec<&str> {
        unimplemented!()
    }

    fn has_multi_select_view(&self) -> bool {
        unimplemented!()
    }

    fn has_single_select_view(&self) -> bool {
        unimplemented!()
    }

    fn has_primary_typecombo_box(&self) -> bool {
        unimplemented!()
    }

    fn has_normal_text_field(&self) -> bool {
        unimplemented!()
    }

    fn has_autocomplete_text_field(&self) -> bool {
        unimplemented!()
    }

    fn has_u64input_field(&self) -> bool {
        unimplemented!()
    }

    fn title(&self) -> &str {
        unimplemented!()
    }

    fn description(&self) -> &str {
        unimplemented!()
    }

    fn apply_label(&self) -> &str {
        unimplemented!()
    }

    fn multi_select_label(&self) -> &str {
        unimplemented!()
    }

    fn single_select_label(&self) -> &str {
        unimplemented!()
    }

    fn combo_box_label(&self) -> &str {
        unimplemented!()
    }

    fn normal_text_field_label(&self) -> &str {
        unimplemented!()
    }

    fn autocomplete_text_field_label(&self) -> &str {
        unimplemented!()
    }

    fn u64input_label(&self) -> &str {
        unimplemented!()
    }

    fn assistant_apply_result_json(&self, json: Value) {
        unimplemented!()
    }
}


fn setup_wizard_for(window_mutex: Mutex<UserWindowGtkComponents>, instance_mutex: Mutex<AssistantDialogue>) {
        //TODO: set labels and data sources via instance

    unimplemented!();

}
