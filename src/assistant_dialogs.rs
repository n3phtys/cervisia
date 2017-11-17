use std::sync::*;

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

    fn assistant_apply_result_json(&self, self, result: AssistantOutput);

}

pub enum AssistantDialogue {
    Donation,
    UserAdministration,
    ItemAdministration
}


pub struct AssistantOutput {
    multiselection: Vec<(&str, u64)>,
    singleselection: Option<(&str, u64)>,
    combobox: &str,
    normaltextfield: &str,
    autocomplete: &str,
    numberOutput: u64,
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


fn setup_wizard_for(window_mutex: Mutex<UserWindowGtkComponents>, instance_mutex: Mutex<AssistantDialogue>) {
        //TODO: set labels and data sources via instance

    unimplemented!();

}
