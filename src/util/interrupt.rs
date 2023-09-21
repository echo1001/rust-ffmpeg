use std::panic;
use std::process;

use ffi::*;
use libc::{c_int, c_void};

pub struct Interrupt {
    pub interrupt: AVIOInterruptCB,
    pub dtor: fn(*mut c_void),
}

impl Drop for Interrupt {
    fn drop(&mut self) {
        (self.dtor)(self.interrupt.opaque);
    }
}

extern "C" fn callback<F>(opaque: *mut c_void) -> c_int
where
    F: FnMut() -> bool,
{
    match panic::catch_unwind(|| (unsafe { &mut *(opaque as *mut F) })()) {
        Ok(ret) => ret as c_int,
        Err(_) => process::abort(),
    }
}

fn destructor<F>(opaque: *mut c_void)
where
    F: FnMut() -> bool,
{
    if opaque.is_null() {
        return;
    }
    match panic::catch_unwind(|| unsafe { drop(Box::from_raw(opaque as *mut F)) }) {
        Err(_) => process::abort(),
        _ => (),
    }
}

pub fn new<F>(opaque: Box<F>) -> Interrupt
where
    F: FnMut() -> bool,
{
    let interrupt_cb = AVIOInterruptCB {
        callback: Some(callback::<F>),
        opaque: Box::into_raw(opaque) as *mut c_void,
    };
    Interrupt {
        interrupt: interrupt_cb,
        dtor: destructor::<F>,
    }
}
