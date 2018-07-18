use libc;

use buf::TCVec;
use std::marker::PhantomData;

#[repr(C)]
pub struct BDBCUR {
    private: [u8; 0]
}

#[link(name="tokyocabinet")]
extern {
    pub fn tcbdbcurdel(cur: *mut BDBCUR);
    pub fn tcbdbcurfirst(cur: *mut BDBCUR) -> bool;
    pub fn tcbdbcurlast(cur: *mut BDBCUR) -> bool;
    pub fn tcbdbcurjump(cur: *mut BDBCUR, kbuf: *const u8, ksiz: libc::c_int) -> bool;
    pub fn tcbdbcurprev(cur: *mut BDBCUR) -> bool;
    pub fn tcbdbcurnext(cur: *mut BDBCUR) -> bool;
    pub fn tcbdbcurput(cur: *mut BDBCUR, vbuf: *const u8, vsiz: libc::c_int, cpmode: libc::c_int) -> bool;
    pub fn tcbdbcurout(cur: *mut BDBCUR) -> bool;
    pub fn tcbdbcurkey(cur: *mut BDBCUR, sp: *mut libc::c_int) -> *mut u8;
    pub fn tcbdbcurval(cur: *mut BDBCUR, sp: *mut libc::c_int) -> *mut u8;
}

pub const BDBCPCURRENT: i32 = 0;                          /* current */
pub const BDBCPBEFORE: i32 = 1;                           /* before */
pub const BDBCPAFTER: i32 = 2;                             /* after */

pub struct Cursor<'a> {
    cur: *mut BDBCUR,
    marker: PhantomData<&'a BDBCUR>,
}

impl<'a> Cursor<'a> {
    pub unsafe fn from_raw<'b>(cur: *mut BDBCUR) -> Cursor<'b> {
        Cursor{ cur: cur, marker: PhantomData }
    }

    pub fn first(&mut self) -> bool {
        unsafe {
            tcbdbcurfirst(self.cur)
        }
    }

    pub fn last(&mut self) -> bool {
        unsafe {
            tcbdbcurlast(self.cur)
        }
    }

    pub fn jump(&mut self, key: &[u8]) -> bool {
        unsafe {
            tcbdbcurjump(self.cur, key.as_ptr(), key.len() as libc::c_int)
        }
    }

    pub fn prev(&mut self) -> bool {
        unsafe {
            tcbdbcurprev(self.cur)
        }
    }

    pub fn next(&mut self) -> bool {
        unsafe {
            tcbdbcurnext(self.cur)
        }
    }

    pub fn put(&mut self, val: &[u8], pos: i32) -> bool {
        unsafe {
            tcbdbcurput(self.cur, val.as_ptr(), val.len() as libc::c_int, pos as libc::c_int)
        }
    }

    pub fn out(&mut self) -> bool {
        unsafe {
            tcbdbcurout(self.cur)
        }
    }

    pub fn key(&self) -> Option<TCVec> {
        unsafe {
            let mut sz: libc::c_int = 0;
            let sp = tcbdbcurkey(self.cur, &mut sz as *mut libc::c_int);
            if sp.is_null() {
                None
            } else {
                Some(TCVec::from_raw(sp, sz))
            }
        }
    }

    pub fn val(&self) -> Option<TCVec> {
        unsafe {
            let mut sz: libc::c_int = 0;
            let sp = tcbdbcurval(self.cur, &mut sz as *mut libc::c_int);
            if sp.is_null() {
                None
            } else {
                Some(TCVec::from_raw(sp, sz))
            }
        }
    }
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        unsafe { tcbdbcurdel(self.cur) }
    }
}
