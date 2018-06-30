#![feature(alloc_system)]
extern crate alloc_system;

extern crate libc;

use std::ffi::CString;
use std::env;
use std::ops::Deref;

#[repr(C)]
pub struct BDB {
    private: [u8; 0]
}

#[link(name="tokyocabinet")]
extern {
    fn tcbdbnew() -> *mut BDB;
    fn tcbdbopen(db: *mut BDB, path: *const libc::c_char, omode: libc::c_int) -> bool;
    fn tcbdbput(db: *mut BDB, kbuf: *const u8, ksiz: libc::c_int, vbuf: *const u8, vsiz: libc::c_int) -> bool;
    fn tcbdbget(db: *mut BDB, kbuf: *const u8, ksiz: libc::c_int, sp: *mut libc::c_int) -> *mut u8;
    fn tcbdbclose(db: *mut BDB) -> bool;
    fn free(p: *const u8);
}

const BDBOREADER: libc::c_int = 1 << 0;                   /* open as a reader */
const BDBOWRITER: libc::c_int = 1 << 1;                   /* open as a writer */
const BDBOCREAT: libc::c_int = 1 << 2;                    /* writer creating */
const BDBOTRUNC: libc::c_int = 1 << 3;                    /* writer truncating */
const BDBONOLCK: libc::c_int = 1 << 4;                    /* open without locking */
const BDBOLCKNB: libc::c_int = 1 << 5;                    /* lock without blocking */
const BDBOTSYNC: libc::c_int = 1 << 6;                    /* synchronize every transaction */

pub struct TCVec {
    size: usize,
    data: *mut u8,
}

impl Deref for TCVec {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.data, self.size)
        }
    }
}

impl Drop for TCVec {
    fn drop(&mut self) {
        unsafe {
            //free(self.data)
        }
    }
}

pub fn sometest() {
    let args: Vec<String> = env::args().collect();

    unsafe {
        let db = tcbdbnew();
        let path = CString::new("test.db").unwrap();

        println!("open returned {}", tcbdbopen(db, path.as_ptr(), BDBOWRITER|BDBOCREAT));

        if args.len() == 3 {
            let ok = tcbdbput(db, args[1].as_ptr(), args[1].len() as libc::c_int, args[2].as_ptr(), args[2].len() as libc::c_int);
            println!("put returned {}", ok);
        } else if args.len() == 2 {
            let mut sz: libc::c_int = 0;
            let sp = tcbdbget(db, args[1].as_ptr(), args[1].len() as libc::c_int, &mut sz as *mut libc::c_int);
            if !sp.is_null() {
                let v = TCVec{ data: sp, size: sz as usize };
                println!("get returned {}", std::str::from_utf8(v.deref()).unwrap());
                //free(sp);
            }
        }

        println!("close returned {}", tcbdbclose(db));
    }
}
