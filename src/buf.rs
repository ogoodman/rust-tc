use std;
use libc;
use std::ops::Deref;

#[link(name="tokyocabinet")]
extern {
    fn tcfree(p: *const u8);
}

pub struct TCVec {
    data: *mut u8,
    size: usize,
}

impl TCVec {
    pub unsafe fn from_raw(data: *mut u8, size: libc::c_int) -> TCVec {
        TCVec{ data: data, size: size as usize }
    }
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
            tcfree(self.data)
        }
    }
}

