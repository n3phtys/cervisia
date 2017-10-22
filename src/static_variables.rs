#![allow(unused_imports)]

use blrustix::*;
use blrustix::build_transient_backend_with;
use blrustix::datastore::Purchase;
use build_purchase_debouncer;
use glade_builders::NUMBER_OF_USERS_PER_PAGE;
use glade_builders::QuickmenuGtkComponents;
use glade_builders::UserWindowGtkComponents;
use glade_builders::build_from_glade;
use glade_builders::build_quickmenu;
use persistencer::TransientPersister;
use rustix_backend::RustixBackend;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

lazy_static! {
    pub static ref GLOBAL_BACKEND:
        Mutex<rustix_backend::RustixBackend<persistencer::TransientPersister>> =
        Mutex::new(build_transient_backend_with(
            NUMBER_OF_USERS_PER_PAGE , NUMBER_OF_USERS_PER_PAGE
            ));
    pub static ref GLOBAL_QUICKMENU
        : Mutex<QuickmenuGtkComponents>
        = Mutex::new(build_quickmenu());
    pub static ref GLOBAL_USERWINDOW
        : Mutex<UserWindowGtkComponents>
        = Mutex::new(build_from_glade());

    pub static ref ADD_OR_UNDO_PURCHASE
        : Mutex<(Sender<Purchase>, Sender<Purchase>)>
        = Mutex::new(build_purchase_debouncer());


    pub static ref USER_SELECTED : Mutex<Option<u32>> = Mutex::new(None);
    pub static ref USERS_ON_SCREEN : Mutex<Vec<u32>> = Mutex::new(Vec::new());
    pub static ref ITEMS_ON_SCREEN : Mutex<Vec<u32>> = Mutex::new(Vec::new());
}
