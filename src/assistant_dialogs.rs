use std::sync::*;
use gtk;
use gdk;
use gtk::prelude::*;
use static_variables::*;
use glade_builders::*;

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

    fn assistant_apply_result_json(&self, result: AssistantOutput);

}

pub enum AssistantDialogue {
    Donation,
    UserAdministration,
    ItemAdministration
}


pub struct AssistantOutput {
    multiselection: Vec<(String, u64)>,
    singleselection: Option<(String, u64)>,
    combobox: String,
    normaltextfield: String,
    autocomplete: String,
    numberoutput: u64,
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

    fn assistant_apply_result_json(&self, result: AssistantOutput) {
        unimplemented!()
    }
}


pub fn setup_util_page_for() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    //window.set_keep_above(true);

    window.set_title("First GTK+ Program");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let button = gtk::Button::new_with_label("Click me!");

    button.connect_clicked(move |_| {
       println!("Button clicked!");
    });

    window.add(&button);

    {
        //set main as parent
        let mainwindow: &gtk::ApplicationWindow = & GLOBAL_USERWINDOW.lock().expect(
            "Global UserWindow variable does not exist anymore",
        ).application_window;

        window.set_transient_for(Some(mainwindow));
        window.set_modal(true);
        window.set_type_hint(gdk::WindowTypeHint::Menu);
        window.set_deletable(false);
    }


    window.show_all();
}


pub fn setup_wizard_for(
    //instance_mutex: Mutex<AssistantDialogue>
    ) {
        //TODO: set labels and data sources via instance

    //let assistant: &mut gtk::Assistant = &mut GLOBAL_USERWINDOW.lock().expect(
    //    "Global UserWindow variable does not exist anymore",
    //).wizard_gen;

    let assistant = gtk::Assistant::new();

    //assistant

    let button = gtk::Button::new_with_label("Click me!");
    let button1 = gtk::Button::new_with_label("Click me 1!");
    let button2 = gtk::Button::new_with_label("Click me 2!");



    assistant.insert_page(&button, 0);
    assistant.set_child_page_type(&button, gtk::AssistantPageType::Intro);
    assistant.insert_page(&button1, 1);
    assistant.set_child_page_type(&button1, gtk::AssistantPageType::Confirm);
    assistant.insert_page(&button1, 2);
    assistant.set_child_page_type(&button2, gtk::AssistantPageType::Summary);

    assistant.set_current_page(0);
    assistant.show();

    //unimplemented!();

}
