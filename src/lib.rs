#![no_std]

use core::{mem::size_of, ptr};

struct Header {
    size: usize,
    is_mmap: usize,
    next: *mut Header,
}

const ALIGN: usize = 8;
const MAX_BYTE: usize = 512;
const INIT_LIST_SIZE: usize = 512;
const ADD_LIST_SIZE: usize = 512;
const NUM_LIST: usize = MAX_BYTE / ALIGN + 1;
const INIT_HEAP_SIZE: usize = NUM_LIST * (INIT_LIST_SIZE + size_of::<Header>());
static mut IS_INIT_MALLOC: bool = false;
static mut FREE_LISTS: [*mut Header; NUM_LIST] = [ptr::null_mut(); (NUM_LIST)];

fn get_align(size: usize) -> usize {
    (size + ALIGN - 1) / ALIGN * ALIGN
}

fn get_header(ptr: *mut ()) -> *mut Header {
    unsafe {
        ptr.sub(size_of::<Header>()) as *mut Header
    }
}

fn init_malloc() -> Result<(), *mut ()> {
    unsafe {
        IS_INIT_MALLOC  = true;

        todo!()
        //let current_ptr = 
    }
}