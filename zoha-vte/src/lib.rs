extern crate gtk;
// #[macro_use]
extern crate glib;
extern crate gdk;
extern crate pango;
extern crate zoha_vte_sys;
extern crate gobject_sys;
extern crate bitflags;
extern crate atk;

#[allow(unused)]
macro_rules! assert_initialized_main_thread {
    () => (
        if !::gtk::is_initialized_main_thread() {
            if ::gtk::is_initialized() {
                panic!("GTK may only be used from the main thread.");
            }
            else {
                panic!("GTK has not been initialized. Call `gtk::init` first.");
            }
        }
    )
}

macro_rules! skip_assert_initialized {
    () => ()
}

mod auto;

pub use auto::*;

