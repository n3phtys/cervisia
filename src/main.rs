extern crate gtk;

use gtk::prelude::*;
#[macro_use] extern crate closet;
extern crate blrustix;

use std::cell::RefCell;

fn main() {
    println!("Hello, world!");

    let backend = RefCell::new(blrustix::default::build_transient_backend()); //TODO: clone() also clones the value, this prevents the kind of use we want

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


    gtk::main();

}
