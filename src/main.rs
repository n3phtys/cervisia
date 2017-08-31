extern crate gtk;

use gtk::prelude::*;
#[macro_use] extern crate closet;
extern crate blrustix;

use std::ops::DerefMut;
use std::cell::RefCell;
use std::rc::Rc;
use blrustix::rustix_backend::*;
use blrustix::persistencer::*;

fn main() {
    println!("Hello, world!");

    let mut backend = Rc::new(RefCell::new(blrustix::build_transient_backend()));

    {
        //prepare transient backup a little bit
        let mut bl2 = &*Rc::get_mut(&mut backend).unwrap();
        let mut bl3 = bl2.borrow_mut();
        let bl : &mut RustixBackend<TransientPersister> = bl3.deref_mut();

        bl.create_user("Gruin".to_string());
        bl.create_user("Vall".to_string());
        bl.create_user("rad(i)".to_string());

        bl.create_item("Club Mate".to_string(), 100, Some("without alcohol".to_string()));
        bl.create_item("Pils".to_string(), 95, Some("Beer".to_string()));
        bl.create_item("Whiskey".to_string(), 1200, Some("Liquor".to_string()));

        bl.purchase(0, 2, 12345678u32);

    }



    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("Cervisia 6.0");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all();

    gtk::main();

}