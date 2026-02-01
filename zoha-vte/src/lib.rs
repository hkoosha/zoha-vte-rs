#![cfg_attr(docsrs, feature(doc_cfg))]

pub use auto::*;
pub use auto::traits::*;
pub use auto::builders::*;

use ffi;
use glib::translate::{Stash, ToGlibContainerFromSlice, ToGlibPtr};
mod auto;

impl<'a> ToGlibContainerFromSlice<'a, *mut *mut ffi::VteRegex> for &Regex {
    type Storage = (
        Vec<Stash<'a, *mut ffi::VteRegex, &'a Regex>>,
        Option<Vec<*mut ffi::VteRegex>>,
    );

    fn to_glib_none_from_slice(t: &'a [&Regex]) -> (*mut *mut ffi::VteRegex, Self::Storage) {
        let stash_vec: Vec<_> = t.iter().map(|r| r.to_glib_none()).collect();
        let mut ptr_vec: Vec<_> = stash_vec.iter().map(|s| s.0).collect();
        ptr_vec.push(std::ptr::null_mut()); // null-terminated

        (ptr_vec.as_ptr() as *mut *mut _, (stash_vec, Some(ptr_vec)))
    }

    fn to_glib_container_from_slice(t: &'a [&Regex]) -> (*mut *mut ffi::VteRegex, Self::Storage) {
        let stash_vec: Vec<_> = t.iter().map(|r| r.to_glib_none()).collect();

        let ptr_vec = unsafe {
            let ptr = glib::ffi::g_malloc(size_of::<*mut ffi::VteRegex>() * (t.len() + 1))
                as *mut *mut ffi::VteRegex;

            for (i, stash) in stash_vec.iter().enumerate() {
                std::ptr::write(ptr.add(i), stash.0);
            }
            std::ptr::write(ptr.add(t.len()), std::ptr::null_mut());
            ptr
        };

        (ptr_vec, (stash_vec, None))
    }

    fn to_glib_full_from_slice(t: &[&Regex]) -> *mut *mut ffi::VteRegex {
        unsafe {
            let ptr = glib::ffi::g_malloc(size_of::<*mut ffi::VteRegex>() * (t.len() + 1))
                as *mut *mut ffi::VteRegex;

            for (i, regex) in t.iter().enumerate() {
                std::ptr::write(ptr.add(i), regex.to_glib_full());
            }
            std::ptr::write(ptr.add(t.len()), std::ptr::null_mut());
            ptr
        }
    }
}
