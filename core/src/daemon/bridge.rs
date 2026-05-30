//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::{cstr, sys};

pub fn notify_service_death(context: &str) {
    let _ = cstr::with_cstr(context, |c_context| unsafe {
        sys::notify_service_death(c_context);
    })
    .map_err(|_| unsafe {
        sys::notify_service_death(c"Service Death".as_ptr());
    });
}
