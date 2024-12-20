use libc::{c_void, free};
use libc::{malloc, size_t};

#[repr(C)]
struct Pixel {
    x: u32,
    y: u32,
    r: u8,
    g: u8,
    b: u8,
}

struct MyIter {
    value: u32,
}

// TODO use std::alloc instead

#[no_mangle]
pub unsafe extern "C" fn iter_create(_arg: *const c_void) -> *mut c_void {
    let res: *mut MyIter = malloc(std::mem::size_of::<MyIter>() as size_t) as *mut MyIter;
    if res.is_null() {
        return std::ptr::null_mut();
    }
    *res = MyIter { value: 0 };
    res as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn iter_destroy(it: *mut c_void) {
    free(it);
}

#[no_mangle]
pub unsafe extern "C" fn iter_next(it: *mut c_void, px: *mut Pixel) -> i32 {
    let my: *mut MyIter = it as *mut MyIter;
    if (*my).value >= 10 {
        return 0;
    }
    *px = Pixel {
        x: (*my).value,
        y: 0,
        r: 255,
        g: 0,
        b: 0,
    };
    (*my).value += 1;
    1
}
