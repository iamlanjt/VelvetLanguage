use std::ffi::CStr;
use std::ffi::c_char;
use std::io::{self, Write};

#[no_mangle]
pub unsafe extern "C" fn vel_write(a: *const c_char) {
    if a.is_null() {
        return;
    }
    print!("{}", CStr::from_ptr(a).to_str().unwrap());
    std::io::stdout().flush().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn vel_print(a: *const c_char) {
    if a.is_null() {
        return;
    }
    print!("{}", CStr::from_ptr(a).to_str().unwrap());
    print!("{}", "\n");
    std::io::stdout().flush().unwrap();
    /*
    unsafe {
        let mut i = 0;
        loop {
            let ch = *a.add(i);
            if ch == 0 {
                break;
            }
            let c_str = unsafe { CStr::from_ptr(ch) };
            print!("{}", ch);
            i += 1;
        }
        std::io::stdout().flush().unwrap();
        println!("");
    }
    */
}
