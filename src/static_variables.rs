#![allow(unused_imports)]

use glade_builders::NUMBER_OF_USERS_PER_PAGE;
use glade_builders::QuickmenuGtkComponents;
use glade_builders::UserWindowGtkComponents;
use glade_builders::build_from_glade;
use glade_builders::build_quickmenu;
use persistencer::TransientPersister;
use rustix_backend::RustixBackend;
use rustix_bl::*;
use rustix_bl::build_transient_backend_with;
use rustix_bl::datastore::Purchase;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};
use std;
use std::collections::HashMap;
use config;
use config::*;
use std::fs::File;
use std::io::Write;
use std::io::Read;


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


    pub static ref USER_SELECTED : Mutex<Option<u32>> = Mutex::new(None);
    pub static ref PURCHASE_SELECTED : Mutex<Option<u64>> = Mutex::new(None);
    pub static ref USERS_ON_SCREEN : Mutex<Vec<u32>> = Mutex::new(Vec::new());
    pub static ref ITEMS_ON_SCREEN : Mutex<Vec<u32>> = Mutex::new(Vec::new());


    pub static ref PROGRAM_CONFIG :HashMap<String, String> = read_config_from_home();

}


pub fn read_config_from_home() -> HashMap<String, String> {
    println!("Reading config!");
    let mut settings = Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name(path_to_config_file_and_mkdirs().to_str().unwrap())).unwrap()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("CERVISIA")).unwrap();

    let config = settings.deserialize::<HashMap<String, String>>().unwrap();

    // Print out our settings (as a HashMap)
    println!("Config: {:?}", config);
    return config;
}


fn path_to_config_file_and_mkdirs() -> std::path::PathBuf {
    let mut path = std::env::home_dir().unwrap();
    path.push(".cervisia");

    {
        let path = path.clone();
        let _ = std::fs::create_dir_all(path);
    }
    {
        path.push("Settings");
        path.set_extension("toml");
    }

    let path2 = path.clone();

    {
        println!("Reading from path {:?}", path);
    }

    let f_opt = File::open(path);

    if f_opt.is_ok() {
        println!("File found in {:?}", path2);
    } else {
        let path3 = path2.clone();
        let mut k = File::create(path3).unwrap();
        let str_incl = include_str!("SettingsDefault.toml");
        k.write_all(
            str_incl.as_bytes()).unwrap();
    }

    return path2;
}

pub const SIZE_OF_PURCHASE_LOG: u16 = 100;
