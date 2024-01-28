#![no_std]

use core::{mem::size_of, ptr};
use libc::{mmap, sbrk, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};

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
    unsafe { ptr.sub(size_of::<Header>()) as *mut Header }
}

fn init_malloc() -> Result<(), *mut ()> {
    unsafe {
        IS_INIT_MALLOC = true;

        let current_ptr = sbrk(0) as *mut ();
        let ret = sbrk((INIT_HEAP_SIZE as isize).try_into().unwrap()) as *mut ();

        if ret != current_ptr {
            return Err(ret);
        }
        let mut p = ret;
        for i in 1..NUM_LIST {
            FREE_LISTS[i] = p as *mut Header;

            let num_header = INIT_LIST_SIZE / (i * ALIGN);
            for j in 0..num_header {
                let header = p as *mut Header;
                let size = i * ALIGN;
                (*header).size = size;
                (*header).is_mmap = 0;
                (*header).next = ptr::null_mut();

                let next_ptr = p.add(size + size_of::<Header>());
                if j != (num_header - 1) {
                    (*header).next = next_ptr as *mut Header;
                } else {
                    (*header).next = ptr::null_mut();
                }

                p = next_ptr;
            }
        }
    }
    Ok(())
}

fn add_list(size: usize) -> Result<*mut Header, *mut ()> {
    unsafe {
        let current_ptr = sbrk(0) as *mut ();
        let num_header = ADD_LIST_SIZE / size;
        let ret = sbrk(
            (num_header * (size + size_of::<Header>()))
                .try_into()
                .unwrap(),
        ) as *mut ();

        if ret != current_ptr {
            return Err(ret);
        }

        let mut p = ret;
        for j in 0..num_header {
            let header = p as *mut Header;
            (*header).size = size;
            (*header).is_mmap = 0;
            (*header).next = ptr::null_mut();

            let next_ptr = p.add(size + size_of::<Header>());
            if j != (num_header - 1) {
                (*header).next = next_ptr as *mut Header;
            } else {
                (*header).next = ptr::null_mut();
            }

            p = next_ptr;
        }

        Ok(ret as *mut Header)
    }
}

fn find_chunk(size: usize) -> Result<*mut Header, *mut ()> {
    unsafe {
        let index = size / 8;

        if FREE_LISTS[index] == ptr::null_mut() {
            let new_list_ret = add_list(size);

            match new_list_ret {
                Ok(new_list) => {
                    FREE_LISTS[index] = new_list;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        let header = FREE_LISTS[index];

        FREE_LISTS[index] = (*header).next;

        Ok(header)
    }
}

fn malloc(size: usize) -> *mut () {
    unsafe {
        if size == 0 {
            return ptr::null_mut();
        }
    
        if !IS_INIT_MALLOC {
            if init_malloc().is_err() {
                return ptr::null_mut();
            }
        }
    
        let size_align = get_align(size);
    
        if size_align <= MAX_BYTE {
            let header_ret = find_chunk(size_align);
            if header_ret.is_err() {
                return ptr::null_mut();
            }
            let header = header_ret.unwrap();
            return (header as *mut ()).add(size_of::<Header>());
        }
    
        let mmap_size = size_of::<Header>() + size;
    
        let p = libc::mmap(
            ptr::null_mut(),
            mmap_size,
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        );
    
        if p == libc::MAP_FAILED {
            return ptr::null_mut();
        }
    
        let header = p as *mut Header;
        (*header).size = mmap_size;
        (*header).is_mmap = 1;
    
        p.add(size_of::<Header>()) as *mut ()
    }
}
