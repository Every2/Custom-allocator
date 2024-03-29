#![no_std]

pub use core::{arch::asm, mem::size_of, ptr};

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
const INITIAL_HEAP_SIZE: usize =
    NUMBER_OF_MEM_BLOCKS * (INITIAL_LIST_MEM_SIZE + size_of::<Header>());
/// Check if memory allocator is already initialized
static mut IS_MALLOC_INITIALIZED: bool = false;
/// Check free memory blocks
static mut AVALIABLE_BLOCKS: [*mut Header; NUMBER_OF_MEM_BLOCKS] =
    [ptr::null_mut(); (NUMBER_OF_MEM_BLOCKS)];

static mut CURRENT_BREAK: *mut () = ptr::null_mut();


const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const PROT_EXEC: i32 = 0x4;
const MAP_ANONYMOUS: i32 = 0x20;
const MAP_PRIVATE: i32 = 0x2;
const MAP_FAILED: *mut u8 = ptr::null_mut();

fn r_brk(address: *mut ()) -> i32 {
    unsafe {
        asm! (
            "mov rax, 0x0C",
            "mov rdi, {}",
            "syscall",
            "mov {}, rax",
            in(reg) address,
            out(reg) CURRENT_BREAK,
        );
        if CURRENT_BREAK < address {
            -1
        } else {
            0
        }
    }
}

fn r_sbrk(increment: isize) -> *mut () {
    unsafe {
        if CURRENT_BREAK.is_null() {
            if r_brk(ptr::null_mut()) < 0 {
                return usize::MAX as *mut ();
            }
        }
        if increment == 0 {
            return CURRENT_BREAK;
        }
        let old_break = CURRENT_BREAK;
        let new_break = (old_break as *mut u8).wrapping_offset(increment) as *mut ();
        match r_brk(new_break) {
            0 => old_break,
            _ => usize::MAX as *mut (),
        }
    }
}

unsafe fn syscall_mmap(
    addr: *mut (),
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: isize,
) -> *mut () {
    let ret: i32;
    asm!(
        "syscall",
        inout("rax") 9 => ret,
        in("rdi") addr,
        in("rsi") length,
        in("rdx") prot,
        in("r10") flags,
        in("r8") fd,
        in("r9") offset,
        lateout("rcx") _,
        lateout("r11") _,
    );
    ret as *mut ()
}

unsafe fn syscall_munmap(addr: *mut (), length: usize) -> i32 {
    let ret: i32;
    asm!(
        "syscall",
        inout("rax") 11 => ret,
        in("rdi") addr,
        in("rsi") length,
        lateout("rcx") _,
        lateout("r11") _,
    );
    ret as i32
}


fn r_munmap(addr: *mut (), length: usize) -> Result<(), i32> {
    
    let result = unsafe { syscall_munmap(addr, length) };

    if result == 0 {
        Ok(())
    } else {
        Err(-1)
    }
}

fn r_mmap(
    addr: *mut (),
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: isize,
) -> Result<*mut (), i32> {
    let addr = unsafe {
        syscall_mmap(addr, length, prot, flags, fd, offset)
    };

    if addr == ptr::null_mut() {
        Err(-1)
    } else {
        Ok(addr)
    }
}


fn get_align(size: usize) -> usize {
    (size + MEMORY_ALIGMENT - 1) / MEMORY_ALIGMENT * MEMORY_ALIGMENT
}

fn get_block_head(pointer: *mut ()) -> *mut Header {
    unsafe { pointer.sub(size_of::<Header>()) as *mut Header }
}

fn init_malloc() -> Result<(), *mut ()> {
    unsafe {
        IS_MALLOC_INITIALIZED = true;

        let current_ptr = r_sbrk(0);
        let allocated_memory_ptr = r_sbrk(INITIAL_HEAP_SIZE as isize);

        if allocated_memory_ptr != current_ptr as *mut () {
            return Err(allocated_memory_ptr as *mut ());
        }

        let mut pointer = allocated_memory_ptr;
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
        let current_ptr = r_sbrk(0) as *mut ();
        let num_header = ADD_LIST_SIZE / size;
        let allocated_memory_ptr = r_sbrk(
            (num_header * (size + size_of::<Header>()))
                .try_into()
                .unwrap(),
        ) as *mut ();

        if allocated_memory_ptr != current_ptr {
            return Err(allocated_memory_ptr);
        }

        let mut pointer = allocated_memory_ptr;
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

        Ok(allocated_memory_ptr as *mut Header)
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
                let current_ptr = r_sbrk(0) as *mut ();
                let sbrk_result =
                    r_sbrk((num_header * (size_align + size_of::<Header>())) as isize);
                if sbrk_result == MAP_FAILED as *mut () {
                    return ptr::null_mut();
                }
                if sbrk_result != current_ptr {
                    return ptr::null_mut();
                }

                let mut pointer = sbrk_result as *mut Header;
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
            let pointer = match r_mmap(ptr::null_mut(), mmap_size, PROT_READ | PROT_WRITE | PROT_EXEC, MAP_ANONYMOUS | MAP_PRIVATE, -1, 0) {
                Ok(pointer) => pointer,
                Err(_) => {
                   return ptr::null_mut()
                }
            };

            if pointer  == ptr::null_mut() {
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
        let current_brk = r_sbrk(0) as *mut ();
        let new_brk = current_brk.offset(total_size as isize);

        if new_brk > (INITIAL_HEAP_SIZE as *mut ()).offset(INITIAL_HEAP_SIZE as isize) {
            return ptr::null_mut();
        }

        r_sbrk(total_size as isize);

        let pointer = current_brk;

        ptr::write_bytes(pointer, 0, total_size);

        pointer
    }
}

fn print(message: &str) -> isize {
    let sys_call_result: i32;
    let ptr_message = message.as_ptr();
    let size: usize = message.len();
    unsafe {
        asm! {
            "syscall",
            inout("rax") 1  => sys_call_result,
            in("rdi") 1,
            in("rsi") ptr_message,
            in("rdx") size,
            lateout("rcx") _,
            lateout("r11") _,
        };
    }
    sys_call_result as isize
}

pub fn free(pointer: *mut ()) {
    unsafe {
        if pointer == ptr::null_mut() {
            return;
        }

        let header = get_block_head(pointer);
        let size = (*header).size;
        if (*header).is_mmap == 1 {
            match r_munmap(pointer.sub(size_of::<Header>()) as *mut (), size) {
                Ok(_) => {
                }
                Err(_) => {
                    let message = "fail munmap\n";
                    print(message);
                }
            }
        } else {
            let index = size / MEMORY_ALIGMENT;
            let first_header = AVALIABLE_BLOCKS[index];
            AVALIABLE_BLOCKS[index] = header;
            (*header).next = first_header;
        }
    }
}

