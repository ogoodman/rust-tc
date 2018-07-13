use libc;
use std;

#[repr(C)]
pub struct TCLIST {
    private: [u8; 0]
}

#[link(name="tokyocabinet")]
extern {
    pub fn tclistnew() -> *mut TCLIST;
    pub fn tclistdup(list: *mut TCLIST) -> *mut TCLIST;
    pub fn tclistdel(list: *mut TCLIST);
    pub fn tclistnum(list: *mut TCLIST) -> libc::c_int;
    pub fn tclistpush(list: *mut TCLIST, ptr: *const u8, size: libc::c_int);
    pub fn tclistval(list: *mut TCLIST, index: libc::c_int, sp: *mut libc::c_int) -> *const u8;
}

pub struct List {
    list: *mut TCLIST,
}

pub struct ListItems<'a> {
    list: &'a List,
    index: usize,
    size: usize,
}

impl<'a> Iterator for ListItems<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<(&'a [u8])> {
        if self.index < self.size {
            let index = self.index;
            self.index += 1;
            Some(&self.list[index])
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = &'a [u8];
    type IntoIter = ListItems<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ListItems{ list: &self, index: 0, size: self.len() }
    }
}

impl List {
    pub unsafe fn from_raw(list: *mut TCLIST) -> List {
        List{ list: list }
    }

    pub fn new() -> List {
        unsafe { List{ list: tclistnew() } }
    }

    pub fn len(&self) -> usize {
        unsafe { tclistnum(self.list) as usize }
    }

    pub fn push(&mut self, val: &[u8]) {
        unsafe { tclistpush(self.list, val.as_ptr(), val.len() as libc::c_int) }
    }
}

impl std::ops::Index<usize> for List {
    type Output = [u8];

    fn index(&self, i: usize) -> &[u8] {
        unsafe {
            let mut size: libc::c_int = 0;
            let data = tclistval(self.list, i as libc::c_int, &mut size as *mut libc::c_int);
            if data.is_null() {
                panic!("TCLIST: index out of range");
            }
            std::slice::from_raw_parts(data, size as usize)
        }
    }
}

impl Drop for List {
    fn drop(&mut self) {
        unsafe {
            tclistdel(self.list)
        }
    }
}
