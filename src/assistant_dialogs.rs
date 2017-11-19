use std::sync::*;
use gtk;
use gdk;
use glib;
use std;
use gtk::prelude::*;
use gtk::*;use gtk::{
    CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn,
    Window, WindowPosition, WindowType
};use static_variables::*;
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

fn create_and_fill_model() -> ListStore {
    // Creation of a model with two rows.
    let model = ListStore::new(&[u32::static_type(), String::static_type(), bool::static_type()]);

    // Filling up the tree view.
    let entries = &["Michel", "Sara", "Liam", "Zelda", "Neo", "Octopus master"];
    let entries_bool = &mut [false, true, true, false, false, true];
    for (i, entry) in entries.iter().enumerate() {
        model.insert_with_values(None, &[0, 1, 2], &[&(i as u32 + 1), &entry, &(entries_bool[i])]);
    }

    model
}

fn append_column(tree: &TreeView, id: i32) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);


    tree.append_column(&column);
}

fn append_checkbox_column(tree: &TreeView, id: i32) {
    let column = TreeViewColumn::new();
    let cell = CellRendererToggle::new();

    cell.set_activatable(true);
    cell.set_sensitive(true);


    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "active", id);

    let mut value: glib::TypedValue<bool> = glib::TypedValue::from(&true);
    //value.set_some(&true);
    cell.set_property("editable", &value);

    let tree2 = tree.clone();

    cell.connect_toggled(move |cellrenderertoggle: &gtk::CellRendererToggle, path: gtk::TreePath|{
        //println!("Being toggled with a = {:?} and b = {:?}!", cellrenderertoggle, path);
        let modelraw = tree2.get_model().unwrap();

        let model = unsafe {
            std::mem::transmute::<TreeModel, ListStore>(modelraw)
        };

        let iter: TreeIter = model.get_iter(&path).unwrap();
        let val0 = model.get_value(&iter, 0);
        let val1 = model.get_value(&iter, 1);
        let val2 = model.get_value(&iter, 2);

        let old_value: bool = model.get_value(&iter, 2).get().unwrap();
        model.set_value(&iter, 2, &glib::TypedValue::from(&(!old_value)));

    });

    tree.append_column(&column);
}

fn create_and_setup_view() -> TreeView {
    // Creating the tree view.
    let tree = TreeView::new();

    tree.set_headers_visible(false);
    // Creating the two columns inside the view.
    append_column(&tree, 0);
    append_column(&tree, 1);
    append_checkbox_column(&tree, 2);
    tree
}



pub fn setup_util_page_for() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    //window.set_keep_above(true);

    window.set_title("First GTK+ Program");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1000, 600);

    let button1 = gtk::Button::new_with_label("Click me 1!");
    let button2 = gtk::Button::new_with_label("Click me 2!");
    let button3 = gtk::Button::new_with_label("Click me 3!");

    button1.connect_clicked(move |_| {
        println!("Button clicked 1!");
    });
    button2.connect_clicked(move |_| {
        println!("Button clicked 2!");
    });
    button3.connect_clicked(move |_| {
        println!("Button clicked 3!");
    });

    let flowbox = gtk::FlowBox::new();
    flowbox.set_max_children_per_line(2);

    flowbox.insert(&button1, -1);
    flowbox.insert(&button2, -1);
    flowbox.insert(&button3, -1);


    {
        let tree = create_and_setup_view();

        //tree.get_selection().set_mode(gtk::SelectionMode::Multiple);

        let model = create_and_fill_model();
        // Setting the model into the view.
        tree.set_model(Some(&model));

        // The closure responds to selection changes by connection to "::cursor-changed" signal,
        // that gets emitted when the cursor moves (focus changes).
        tree.connect_cursor_changed(move |tree_view| {
            let selection = tree_view.get_selection();
            if let Some((model, iter)) = selection.get_selected() {
                // Now getting back the values from the row corresponding to the
                // iterator `iter`.
                //
                // The `get_value` method do the conversion between the gtk type and Rust.
                println!("Hello '{}' from row {}",
                                        model.get_value(&iter, 1)
                                            .get::<String>()
                                            .expect("Couldn't get string value"),
                                        model.get_value(&iter, 0)
                                            .get::<u32>()
                                            .expect("Couldn't get u32 value"));
            }
        });

        // Adding the layout to the window.
        flowbox.insert(&tree, -1);

    }




    window.add(&flowbox);





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
