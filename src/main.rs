extern crate gio;
extern crate glib;
extern crate gtk;

use gtk::prelude::*;
use gtk::{
    CellRendererText, Label, ListStore, Orientation, TreeView, TreeViewColumn,
    Window, WindowPosition, WindowType
};

use gtk::{Builder, Button, MessageDialog};

use gio::prelude::*;
use gtk::{
    AboutDialog, AboutDialogExt, BoxExt, ContainerExt, DialogExt, GtkApplicationExt,
    Inhibit, LabelExt, SwitchExt, ToVariant, WidgetExt
};
use std::env;


#[macro_use]
extern crate closet;
extern crate blrustix;

use std::ops::DerefMut;
use std::cell::RefCell;
use std::rc::Rc;
use blrustix::rustix_backend::*;
use blrustix::persistencer::*;
use blrustix::datastore::*;
use std::cell::*;
use std::ops::*;
use std::borrow::*;
use gtk::ScrolledWindow;
use gtk::Adjustment;


// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}



fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    println!("Hello, world!");

    let mut backend = Rc::new(RefCell::new(blrustix::build_transient_backend()));

    {
        //prepare transient backup a little bit
        let mut bl2 = &*Rc::get_mut(&mut backend).unwrap();
        let mut bl3 = bl2.borrow_mut();
        let bl: &mut RustixBackend<TransientPersister> = bl3.deref_mut();

        bl.create_user("Gruin".to_string());
        bl.create_user("Vall".to_string());
        bl.create_user("rad(i)".to_string());

        for i in 0..99 {
            bl.create_user("GenUser #".to_string() + &i.to_string());
        }

        bl.create_item("Club Mate".to_string(), 100, Some("without alcohol".to_string()));
        bl.create_item("Pils".to_string(), 95, Some("Beer".to_string()));
        bl.create_item("Whiskey".to_string(), 1200, Some("Liquor".to_string()));

        bl.purchase(0, 2, 12345678u32);
    }


    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }


    window.set_title("Cervisia 6.0");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    window.connect_delete_event(|_, _| {
        //gtk::main_quit();
        Inhibit(false)
    });


    // Creating a vertical layout to place both tree view and label in the window.
    let vertical_layout = gtk::Box::new(Orientation::Vertical, 0);

    // Creation of the label.
    let label = Label::new(None);

    let tree = create_and_setup_view();


    let model = {
        let bl2 = &*Rc::get_mut(&mut backend).unwrap();
        let bl3 = bl2.borrow();
        let bl: &RustixBackend<TransientPersister> = bl3.deref();

        create_and_fill_model(&bl.datastore)
    };

    // Setting the model into the view.
    tree.set_model(Some(&model));
    // Adding the view to the layout.

    let scroll = {
        let hadjustment = None;
        let vadjustment = None;
        let wdw = ScrolledWindow::new(hadjustment, vadjustment);
        wdw
    };

    vertical_layout.add(&scroll);

    {
        scroll.add(&tree);
    }

    // Same goes for the label.
    vertical_layout.add(&label);

    // The closure responds to selection changes by connection to "::cursor-changed" signal,
    // that gets emitted when the cursor moves (focus changes).
    tree.connect_cursor_changed(move |tree_view| {
        let selection = tree_view.get_selection();
        if let Some((model, iter)) = selection.get_selected() {
            // Now getting back the values from the row corresponding to the
            // iterator `iter`.
            //
            // The `get_value` method do the conversion between the gtk type and Rust.
            label.set_text(&format!("Hello '{}' from row {}",
                                    model.get_value(&iter, 1).get::<String>().unwrap(),
                                    model.get_value(&iter, 0).get::<u32>().unwrap()));
        }
    });


    // Adding the layout to the window.
    window.add(&vertical_layout);


    add_application_actions(application, &window);

    window.show_all();
}

struct QuickmenuGtkComponents {
    quickmenu: gtk::Dialog,
    item_btn: [gtk::Button; 4],
    other_drinks: gtk::Button,
    free_be: gtk::Button,
    statistics: gtk::Button,
}

fn build_quickmenu() -> QuickmenuGtkComponents {
    let glade_src = include_str!("quickmenu.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::Dialog = builder.get_object("quickmenu").expect("Couldn't get quickmenu");


    let close_btn: gtk::Button = builder.get_object("close_dialog").expect("Couldn't get quickmenu");

    close_btn.connect_clicked(move |_| {
        window.hide();
    });



    return QuickmenuGtkComponents{
        quickmenu: builder.get_object("quickmenu").expect("Couldn't get quickmenu"),
        item_btn: [builder.get_object("item_btn_0").expect("Couldn't get item_btn_0") , builder.get_object("item_btn_1").expect("Couldn't get item_btn_1"), builder.get_object("item_btn_2").expect("Couldn't get item_btn_2"), builder.get_object("item_btn_3").expect("Couldn't get item_btn_3")],
        other_drinks: builder.get_object("andere_getraenke").expect("Couldn't get andere_getraenke"),
        free_be: builder.get_object("ausgeben").expect("Couldn't get ausgeben"),
        statistics: builder.get_object("statistik").expect("Couldn't get statistik"),
    };
}


fn show_quickmenu(quickmenu: &mut QuickmenuGtkComponents) {
    //TODO: parameters like item strings, item id, and both in 4 options total
    //TODO: user id and user name


    quickmenu.quickmenu.show_all();
}

fn build_from_glade() {

    let glade_src = include_str!("main-window.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: gtk::ApplicationWindow = builder.get_object("user_selection_window").expect("Couldn't get user_selection_window");

    window.show_all();
}


fn main() {
    let application = gtk::Application::new("cervisia.gtk", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");




    {
        let app2 = application.clone();

        application.connect_startup(move |app| {
            build_ui(app);



            build_from_glade();
            let mut quickmenu = build_quickmenu();
            show_quickmenu(&mut quickmenu);


            let result_of_registration = app2.register(None).expect("Registration failed");




        });


    }

    {
        let app2 = application.clone();

        application.connect_activate(move |_| {
            {
                let notification_1 = gio::Notification::new("my notification title");

                notification_1.set_body("my notification body with some content");


                notification_1.add_button("My Button", "app.id-notification-undo");

                println!("Sending Notification");

                app2.send_notification("my-notification-id", &notification_1);

                println!("Sent Notification");
            }
        });
    }




    let a: &[&str] = &[];

    std::process::exit(application.run(a));
}


fn create_and_setup_view() -> TreeView {
    // Creating the tree view.
    let tree = TreeView::new();

    tree.set_headers_visible(false);
    // Creating the two columns inside the view.
    append_column(&tree, 0);
    append_column(&tree, 1);
    tree
}


fn append_column(tree: &TreeView, id: i32) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);
    tree.append_column(&column);
}

fn create_and_fill_model(datastore: &Datastore) -> ListStore {
    // Creation of a model with two rows.
    let model = ListStore::new(&[u32::static_type(), String::static_type()]);

    // Filling up the tree view.
    for (_, (id, user)) in datastore.users.iter().enumerate() {
        model.insert_with_values(None, &[0, 1], &[&id, &user.username]);
    }
    model
}


fn add_application_actions(application: &gtk::Application, window: &gtk::ApplicationWindow) {
    let id_notification_undo = gio::SimpleAction::new("id-notification-undo", /*Some(glib::VariantTy::new("int"))*/ None);
    id_notification_undo.connect_activate(clone!(window => move |a, b| {
        println!("Something something");
        println!("Received Action with a = {:?} and b = {:?}", a, b);
    }));
    id_notification_undo.set_enabled(true);

    application.add_action(&id_notification_undo);
}