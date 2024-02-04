use custom_allocator;
use libc::sbrk;

fn main() {
    let ptr1 = custom_allocator::malloc(10);
    let ptr2 = custom_allocator::calloc(5, 10);
    let ptr3 = custom_allocator::realloc(ptr1, 20);
    println!("{:?}", ptr1);
    println!("{:?}", ptr2);
    println!("{:?}", ptr3);
    custom_allocator::free(ptr1);
    custom_allocator::free(ptr2);
    custom_allocator::free(ptr3);
}
