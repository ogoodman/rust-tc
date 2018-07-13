
#![cfg_attr(feature = "nightly", feature(system_allocator))]

#[cfg(system_allocator)]
use std::alloc::System;

#[cfg(system_allocator)]
#[global_allocator]
static GLOBAL: System = System;

extern crate libc;

pub mod list;
pub mod buf;
pub mod bdbcur;

use std::ffi::{CString, CStr};
use std::result;
use std::fmt;
use list::{TCLIST, List};
use buf::TCVec;
use bdbcur::{BDBCUR, Cursor};

pub trait Cmp {
    fn cmp(&self, a: &[u8], b: &[u8]) -> i32;
}

type CMPf = extern fn(*const u8, libc::c_int, *const u8, libc::c_int, *const libc::c_void) -> libc::c_int;

#[repr(C)]
pub struct TCBDB {
    private: [u8; 0]
}

#[link(name="tokyocabinet")]
extern {
    fn tcbdberrmsg(ecode: libc::c_int) -> *const libc::c_char;
    fn tcbdbnew() -> *mut TCBDB;
    fn tcbdbdel(db: *mut TCBDB);
    fn tcbdbecode(db: *mut TCBDB) -> libc::c_int;
    fn tcbdbsetmutex(db: *mut TCBDB) -> bool;
    fn tcbdbsetcmpfunc(db: *mut TCBDB, cmp: CMPf, op: *const libc::c_void) -> bool;

    fn tcbdbtune(db: *mut TCBDB, lmemb: libc::int32_t, nmemb: libc::int32_t, bnum: libc::int64_t, apow: i8, fpow: i8, opts: u8) -> bool;
    fn tcbdbsetcache(db: *mut TCBDB, lcnum: libc::int32_t, ncnum: libc::int32_t) -> bool;
    fn tcbdbsetxmsiz(db: *mut TCBDB, xmsiz: libc::int64_t) -> bool;
    fn tcbdbsetdfunit(db: *mut TCBDB, dfunit: libc::int32_t) -> bool;

    fn tcbdbopen(db: *mut TCBDB, path: *const libc::c_char, omode: libc::c_int) -> bool;
    fn tcbdbclose(db: *mut TCBDB) -> bool;

    fn tcbdbput(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int, vbuf: *const u8, vsiz: libc::c_int) -> bool;
    fn tcbdbputkeep(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int, vbuf: *const u8, vsiz: libc::c_int) -> bool;
    fn tcbdbputcat(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int, vbuf: *const u8, vsiz: libc::c_int) -> bool;
    fn tcbdbputdup(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int, vbuf: *const u8, vsiz: libc::c_int) -> bool;
    fn tcbdbget(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int, sp: *mut libc::c_int) -> *mut u8;
    fn tcbdbvnum(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int) -> libc::c_int;
    fn tcbdbvsiz(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int) -> libc::c_int;

    fn tcbdbrange(db: *mut TCBDB, bkbuf: *const u8, bksiz: libc::c_int, binc: bool, ekbuf: *const u8, eksiz: libc::c_int, einc: bool, max: libc::c_int) -> *mut TCLIST;
    fn tcbdbfwmkeys(db: *mut TCBDB, pbuf: *const u8, psiz: libc::c_int, max: libc::c_int) -> *mut TCLIST;

    fn tcbdbaddint(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int, num: libc::c_int) -> libc::c_int;
    fn tcbdbadddouble(db: *mut TCBDB, kbuf: *const u8, ksiz: libc::c_int, num: libc::c_double) -> libc::c_double;
    fn tcbdbsync(db: *mut TCBDB) -> bool;
    fn tcbdboptimize(db: *mut TCBDB, lmemb: libc::int32_t, nmemb: libc::int32_t, bnum: libc::int64_t, apow: i8, fpow: i8, opts: u8) -> bool;
    fn tcbdbvanish(db: *mut TCBDB) -> bool;
    fn tcbdbcopy(db: *mut TCBDB, path: *const libc::c_char) -> bool;
    fn tcbdbtranbegin(db: *mut TCBDB) -> bool;
    fn tcbdbtrancommit(db: *mut TCBDB) -> bool;
    fn tcbdbtranabort(db: *mut TCBDB) -> bool;
    fn tcbdbpath(db: *mut TCBDB) -> *const libc::c_char;
    fn tcbdbrnum(db: *mut TCBDB) -> libc::uint64_t;
    fn tcbdbfsiz(db: *mut TCBDB) -> libc::uint64_t;

    fn tcbdbcurnew(db: *mut TCBDB) -> *mut BDBCUR;

    pub fn tccmplexical(ap: *const u8, asz: libc::c_int, bp: *const u8, bsz: libc::c_int, op: *const libc::c_void) -> libc::c_int;
}

pub extern "C" fn tccmpcustom(ap: *const u8, asz: libc::c_int, bp: *const u8, bsz: libc::c_int, op: *const libc::c_void) -> libc::c_int {
    unsafe {
        let op = (op as *const Box<Cmp>).as_ref().unwrap();
        let a = std::slice::from_raw_parts(ap, asz as usize);
        let b = std::slice::from_raw_parts(bp, bsz as usize);
        op.cmp(a, b)
    }
}

fn new_cmp_ptr(cmp: Box<Cmp>) -> *mut libc::c_void {
    Box::into_raw(Box::new(cmp)) as *mut libc::c_void
}

unsafe fn del_cmp_ptr(p: *mut libc::c_void) {
    Box::from_raw(p as *mut Box<Cmp>);
}

pub const BDBOREADER: u32 = 1 << 0;                   /* open as a reader */
pub const BDBOWRITER: u32 = 1 << 1;                   /* open as a writer */
pub const BDBOCREAT: u32 = 1 << 2;                    /* writer creating */
pub const BDBOTRUNC: u32 = 1 << 3;                    /* writer truncating */
pub const BDBONOLCK: u32 = 1 << 4;                    /* open without locking */
pub const BDBOLCKNB: u32 = 1 << 5;                    /* lock without blocking */
pub const BDBOTSYNC: u32 = 1 << 6;                    /* synchronize every transaction */

#[derive(Copy,Clone)]
pub struct BDBErr {
    ecode: libc::c_int,
}

impl fmt::Debug for BDBErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = unsafe { tcbdberrmsg(self.ecode) };
        if !p.is_null() {
            unsafe {
                let cs = CStr::from_ptr(p).to_string_lossy();
                write!(f, "BDBErr {}: {}", self.ecode as i32, cs)
            }
        } else {
            write!(f, "BDBErr {}: ?", self.ecode)
        }
    }
}

pub type Result<T> = result::Result<T, BDBErr>;

pub struct BDB {
    db: *mut TCBDB,
    cmp: *mut libc::c_void,
}

impl BDB {
    pub fn new() -> BDB {
        unsafe {
            BDB{ db: tcbdbnew(), cmp: std::ptr::null_mut() }
        }
    }

    pub fn res(&self, ok: bool) -> Result<()> {
        if ok {
            Ok(())
        } else {
            Err(BDBErr{ ecode: unsafe { tcbdbecode(self.db) } })
        }
    }

    pub fn setmutex(&mut self) -> Result<()> {
        self.res(unsafe { tcbdbsetmutex(self.db) })
    }

    pub fn setcmp(&mut self, cmp: Box<Cmp>) -> Result<()> {
        if !self.cmp.is_null() {
            unsafe { del_cmp_ptr(self.cmp) };
        }
        self.cmp = new_cmp_ptr(cmp);
        self.res(unsafe { tcbdbsetcmpfunc(self.db, tccmpcustom, self.cmp) })
    }

    pub fn tune(&mut self, lmemb: i32, nmemb: i32, bnum: i64, apow: i8, fpow: i8, opts: u8) -> Result<()> {
        self.res(unsafe { tcbdbtune(self.db, lmemb as libc::int32_t, nmemb as libc::int32_t, bnum as libc::int64_t, apow, fpow, opts) })
    }

    pub fn setcache(&mut self, lcnum: i32, ncnum: i32) -> Result<()> {
        self.res(unsafe { tcbdbsetcache(self.db, lcnum as libc::int32_t, ncnum as libc::int32_t) })
    }

    pub fn setxmsiz(&mut self, xmsiz: libc::int64_t) -> Result<()> {
        self.res(unsafe { tcbdbsetxmsiz(self.db, xmsiz as libc::int64_t) })
    }

    pub fn setdfunit(&mut self, dfunit: libc::int32_t) -> Result<()> {
        self.res(unsafe { tcbdbsetdfunit(self.db, dfunit as libc::int32_t) })
    }

    pub fn open(&mut self, path: &str, omode: u32) -> Result<()> {
        let cpath = CString::new(path).unwrap();
        self.res(unsafe { tcbdbopen(self.db, cpath.as_ptr(), omode as libc::c_int) })
    }

    pub fn close(&mut self) -> Result<()> {
        self.res(unsafe { tcbdbclose(self.db) })
    }

    pub fn put(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        self.res(unsafe {
            tcbdbput(self.db, key.as_ptr(), key.len() as libc::c_int, val.as_ptr(), val.len() as libc::c_int)
        })
    }

    pub fn putkeep(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        self.res(unsafe {
            tcbdbputkeep(self.db, key.as_ptr(), key.len() as libc::c_int, val.as_ptr(), val.len() as libc::c_int)
        })
    }

    pub fn putcat(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        self.res(unsafe {
            tcbdbputcat(self.db, key.as_ptr(), key.len() as libc::c_int, val.as_ptr(), val.len() as libc::c_int)
        })
    }

    pub fn putdup(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        self.res(unsafe {
            tcbdbputdup(self.db, key.as_ptr(), key.len() as libc::c_int, val.as_ptr(), val.len() as libc::c_int)
        })
    }

    pub fn get(&self, key: &[u8]) -> Option<TCVec> {
        unsafe {
            let mut sz: libc::c_int = 0;
            let sp = tcbdbget(self.db, key.as_ptr(), key.len() as libc::c_int, &mut sz as *mut libc::c_int);
            if sp.is_null() {
                None
            } else {
                Some(TCVec::from_raw(sp, sz))
            }
        }
    }

    pub fn vnum(&self, key: &[u8]) -> u32 {
        unsafe {
            tcbdbvnum(self.db, key.as_ptr(), key.len() as libc::c_int) as u32
        }
    }

    pub fn vsiz(&self, key: &[u8]) -> u32 {
        unsafe {
            tcbdbvsiz(self.db, key.as_ptr(), key.len() as libc::c_int) as u32
        }
    }

    pub fn range(&self, bk: &[u8], binc: bool, ek: &[u8], einc: bool, max: i32) -> List {
        unsafe {
            let list = tcbdbrange(self.db, bk.as_ptr(), bk.len() as libc::c_int, binc, ek.as_ptr(), ek.len() as libc::c_int, einc, max as libc::c_int);
            List::from_raw(list)
        }
    }

    pub fn fwmkeys(&self, prefix: &[u8], max: i32) -> List {
        unsafe {
            let list = tcbdbfwmkeys(self.db, prefix.as_ptr(), prefix.len() as libc::c_int, max as libc::c_int);
            List::from_raw(list)
        }
    }

    pub fn addint(&mut self, key: &[u8], num: i32) -> libc::c_int {
        unsafe {
            tcbdbaddint(self.db, key.as_ptr(), key.len() as libc::c_int, num as libc::c_int) as i32
        }
    }

    pub fn adddouble(&mut self, key: &[u8], num: f64) -> f64 {
        unsafe {
            tcbdbadddouble(self.db, key.as_ptr(), key.len() as libc::c_int, num as libc::c_double) as f64
        }
    }

    pub fn sync(&mut self) -> Result<()> {
        self.res(unsafe { tcbdbsync(self.db) })
    }

    pub fn optimize(&mut self, lmemb: i32, nmemb: i32, bnum: i64, apow: i8, fpow: i8, opts: u8) -> Result<()> {
        self.res(unsafe { tcbdboptimize(self.db, lmemb as libc::int32_t, nmemb as libc::int32_t, bnum as libc::int64_t, apow, fpow, opts) }) 
    }

    pub fn vanish(&mut self) -> Result<()> {
        self.res(unsafe { tcbdbvanish(self.db) }) 
    }

    pub fn copy(&self, path: &str) -> Result<()> {
        let cpath = CString::new(path).unwrap();
        self.res(unsafe { tcbdbcopy(self.db, cpath.as_ptr()) }) 
    }

    pub fn tranbegin(&mut self) -> Result<()> {
        self.res(unsafe { tcbdbtranbegin(self.db) }) 
    }

    pub fn trancommit(&mut self) -> Result<()> {
        self.res(unsafe { tcbdbtrancommit(self.db) }) 
    }

    pub fn tranabort(&mut self) -> Result<()> {
        self.res(unsafe { tcbdbtranabort(self.db) }) 
    }

    pub fn path(&self) -> Option<String> {
        unsafe {
            let p = tcbdbpath(self.db);
            if !p.is_null() {
                Some(CStr::from_ptr(p).to_string_lossy().to_string())
            } else {
                None
            }
        }
    }

    pub fn rnum(&self) -> usize {
        unsafe { tcbdbrnum(self.db) as usize }
    }

    pub fn fsiz(&self) -> usize {
        unsafe { tcbdbfsiz(self.db) as usize }
    }

    pub fn cursor(&self) -> Cursor {
        unsafe {
            Cursor::from_raw(tcbdbcurnew(self.db))
        }
    }
}

impl Drop for BDB {
    fn drop(&mut self) {
        unsafe {
            if !self.cmp.is_null() {
                del_cmp_ptr(self.cmp);
            }
            tcbdbdel(self.db)
        }
    }
}

/*
struct MyCmp {
}

impl Cmp for MyCmp {
    fn cmp(&self, a: &[u8], b: &[u8]) -> i32 {
        42
    }
}

pub fn cmptest() {
    let cp = new_cmp_ptr(Box::new(MyCmp{}));

    let a: &[u8] = "a".as_ref();
    let b: &[u8] = "b".as_ref();

    println!("{}", tccmpcustom(a.as_ptr(), 1, b.as_ptr(), 1, cp));
    unsafe { del_cmp_ptr(cp) };
}
*/

