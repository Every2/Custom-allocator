#![no_std]

use core::{mem::size_of, ptr};
use libc::{
    c_void, mmap, munmap, sbrk, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ,
    PROT_WRITE,
};

struct Header {
    size: usize,
    /// Indicates if the block was allocated via mmap (1 if yes, 0 if no).
    is_mmap: usize,
    next: *mut Header,
}

///Memory alignment (in bytes).
const MEMORY_ALIGMENT: usize = 8;
/// Maximum size of the memory block to be managed in the heap.
const MAX_HEAP_SIZE: usize = 512;
/// Initial size of the list of memory blocks.
const INITIAL_LIST_MEM_SIZE: usize = 512;
/// Size to be added to the list of memory blocks.
const ADD_LIST_SIZE: usize = 512;
///  Number of lists of memory blocks (calculated based on MAX_HEAP_SIZE and MEMORY_ALIGMENT).
const NUMBER_OF_MEM_BLOCKS: usize = MAX_HEAP_SIZE / MEMORY_ALIGMENT + 1;
/// Initial size of the heap
const INITIAL_HEAP_SIZE: usize = NUMBER_OF_MEM_BLOCKS * (INITIAL_LIST_MEM_SIZE + size_of::<Header>());
/// Check if memory allocator is already initialized
static mut IS_MALLOC_INITIALIZED: bool = false;
/// Check free memory blocks
static mut AVALIABLE_BLOCKS: [*mut Header; NUMBER_OF_MEM_BLOCKS] = [ptr::null_mut(); (NUMBER_OF_MEM_BLOCKS)];

fn get_align(size: usize) -> usize {
    (size + MEMORY_ALIGMENT - 1) / MEMORY_ALIGMENT * MEMORY_ALIGMENT
}

fn get_block_head(pointer: *mut ()) -> *mut Header {
    unsafe { pointer.sub(size_of::<Header>()) as *mut Header }
}

fn init_malloc() -> Result<(), *mut ()> {
    unsafe {
        IS_MALLOC_INITIALIZED = true;

        let current_ptr = sbrk(0) as *mut ();
        let ret = sbrk((INITIAL_HEAP_SIZE as isize).try_into().unwrap()) as *mut ();

        if ret != current_ptr {
            return Err(ret);
        }
        let mut pointer = ret;
        for i in 1..NUMBER_OF_MEM_BLOCKS {
            AVALIABLE_BLOCKS[i] = pointer as *mut Header;

            let num_header = INITIAL_LIST_MEM_SIZE / (i * MEMORY_ALIGMENT);
            for j in 0..num_header {
                let header = pointer as *mut Header;
                let size = i * MEMORY_ALIGMENT;
                (*header).size = size;
                (*header).is_mmap = 0;
                (*header).next = ptr::null_mut();

                let next_ptr = pointer.add(size + size_of::<Header>());
                if j != (num_header - 1) {
                    (*header).next = next_ptr as *mut Header;
                } else {
                    (*header).next = ptr::null_mut();
                }

                pointer = next_ptr;
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

        let mut pointer = ret;
        for j in 0..num_header {
            let header = pointer as *mut Header;
            (*header).size = size;
            (*header).is_mmap = 0;
            (*header).next = ptr::null_mut();

            let next_ptr = pointer.add(size + size_of::<Header>());
            if j != (num_header - 1) {
                (*header).next = next_ptr as *mut Header;
            } else {
                (*header).next = ptr::null_mut();
            }

            pointer = next_ptr;
        }

        Ok(ret as *mut Header)
    }
}

fn find_free_mem_block(size: usize) -> Result<*mut Header, *mut ()> {
    unsafe {
        let index = size / 8;

        if AVALIABLE_BLOCKS[index] == ptr::null_mut() {
            let new_list_ret = add_list(size);

            match new_list_ret {
                Ok(new_list) => {
                    AVALIABLE_BLOCKS[index] = new_list;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        let header = AVALIABLE_BLOCKS[index];

        AVALIABLE_BLOCKS[index] = (*header).next;

        Ok(header)
    }
}

pub fn malloc(size: usize) -> *mut () {
    unsafe {
        if !IS_MALLOC_INITIALIZED {
            if init_malloc().is_err() {
                return ptr::null_mut();
            }
        }

        let size_align = (size + MEMORY_ALIGMENT - 1) / MEMORY_ALIGMENT * MEMORY_ALIGMENT;

        if size_align <= MAX_HEAP_SIZE {
            let index = size_align / MEMORY_ALIGMENT - 1;

            if AVALIABLE_BLOCKS[index].is_null() {
                let num_header = ADD_LIST_SIZE / size_align;
                let current_ptr = sbrk(0) as *mut ();
                let ret = sbrk((num_header * (size_align + size_of::<Header>())) as isize);
                if ret == MAP_FAILED {
                    return ptr::null_mut();
                }
                if ret != current_ptr as *mut c_void {
                    return ptr::null_mut();
                }

                let mut pointer = ret as *mut Header;
                for _ in 0..num_header {
                    (*pointer).size = size_align;
                    (*pointer).is_mmap = 0;
                    (*pointer).next = AVALIABLE_BLOCKS[index];
                    AVALIABLE_BLOCKS[index] = pointer;
                    pointer = pointer.offset(1);
                }
            }

            if let Ok(header) = find_free_mem_block(size_align) {
                return (header).add(size_of::<Header>()) as *mut ();
            } else {
                ptr::null_mut()
            }
        } else {
            let mmap_size = size + size_of::<Header>();
            let pointer = mmap(
                ptr::null_mut(),
                mmap_size,
                PROT_READ | PROT_WRITE | PROT_EXEC,
                MAP_ANONYMOUS | MAP_PRIVATE,
                -1,
                0,
            );
            if pointer == MAP_FAILED {
                return ptr::null_mut();
            }
            let header = pointer as *mut Header;
            (*header).size = mmap_size;
            (*header).is_mmap = 1;
            pointer.offset(size_of::<Header>() as isize) as *mut ()
        }
    }
}

pub fn realloc(pointer: *mut (), size: usize) -> *mut () {
    unsafe {
        let size_align = get_align(size);
        if pointer == ptr::null_mut() {
            return malloc(size_align);
        }

        let new_ptr = malloc(size_align);
        let header = get_block_head(pointer);

        let copy_size = if (*header).size < size_align {
            (*header).size
        } else {
            size_align
        };

        let old_ptr = pointer;
        let destination_ptr = new_ptr;

        for i in 0..copy_size {
            *destination_ptr.offset(i as isize) = *old_ptr.offset(i as isize);
        }

        free(pointer);
        new_ptr
    }
}

pub fn calloc(number: usize, size: usize) -> *mut () {
    unsafe {
        let total_size = number * size;
        let current_brk = sbrk(0) as *mut ();
        let new_brk = current_brk.offset(total_size as isize);

        if new_brk > (INITIAL_HEAP_SIZE as *mut ()).offset(INITIAL_HEAP_SIZE as isize) {
            return ptr::null_mut();
        }

        sbrk(total_size as isize);

        let pointer = current_brk;

        ptr::write_bytes(pointer, 0, total_size);

        pointer
    }
}

pub fn free(pointer: *mut ()) {
    unsafe {
        if pointer == ptr::null_mut() {
            return;
        }

        let header = get_block_head(pointer);
        let size = (*header).size;
        if (*header).is_mmap == 1 {
            let nummap_ret = munmap(pointer.sub(size_of::<Header>()) as *mut c_void, size);
            debug_assert!(nummap_ret == 0);
            if nummap_ret == 0 {
            } else {
                let message = "fail munmanp\n";
                let buffer = message.as_ptr() as *const c_void;
                let buffer_len = message.len();
                libc::write(1, buffer, buffer_len);
            }
        } else {
            let index = size / MEMORY_ALIGMENT;
            let first_header = AVALIABLE_BLOCKS[index];
            AVALIABLE_BLOCKS[index] = header;
            (*header).next = first_header;
        }
    }
}
