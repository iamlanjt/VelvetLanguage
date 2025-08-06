use std::io::{self, Write};

#[no_mangle]
pub extern "C" fn rs_print(a: *const u8) {
    /*
    if a.is_null() {
        return;
    }

    unsafe {
        let mut i = 0;
        loop {
            let ch = *a.add(i);
            if ch == 0 {
                break;
            }
            print!("{}", ch as char);
            i += 1;
        }
        io::stdout().flush().unwrap();
    }
    */
}

#[no_mangle]
pub extern "C" fn rs_println(a: *const u8) {
    rs_print(a);
}
