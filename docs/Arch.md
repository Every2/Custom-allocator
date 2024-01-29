# Memory Allocation Documentation

## Structures

### `Header`
- **Description:** Structure representing the header of each allocated memory block.
- `size`: Size of the memory block.
- `is_mmap`: Indicates if the block was allocated via `mmap` (1 if yes, 0 if no).
- `next`: Pointer to the next memory block in the free list.

## Constants

- `MEMORY_ALIGMENT`: Memory alignment (in bytes).
- `MAX_HEAP_SIZE`: Maximum size of the memory block to be managed in the heap.
- `INITIAL_LIST_MEM_SIZE`: Initial size of the list of memory blocks.
- `ADD_LIST_SIZE`: Size to be added to the list of memory blocks.
- `NUMBER_OF_MEM_BLOCKS`: Number of lists of memory blocks (calculated based on `MAX_HEAP_SIZE` and `MEMORY_ALIGMENT`).
- `INITIAL_HEAP_SIZE`: Initial size of the heap.

## Functions

### `malloc(size: usize) -> *mut ()`
- **Description:** Allocates a memory block of size `size`.
- **Parameters:**
  - `size`: Size of the memory block to be allocated.
- **Returns:** Pointer to the allocated memory block.

### `realloc(pointer: *mut (), size: usize) -> *mut ()`
- **Description:** Changes the size of the memory block pointed to by `pointer` to `size`.
- **Parameters:**
  - `pointer`: Pointer to the memory block to be reallocated.
  - `size`: New size of the memory block.
- **Returns:** Pointer to the reallocated memory block.

### `calloc(number: isize, size: isize) -> *mut ()`
- **Description:** Allocates and initializes a memory block for an array of `number` elements of `size` bytes each.
- **Parameters:**
  - `number`: Number of elements in the array.
  - `size`: Size in bytes of each element.
- **Returns:** Pointer to the allocated memory block initialized with zeros.

### `free(pointer: *mut ())`
- **Description:** Frees the memory block pointed to by `pointer`.
- **Parameters:**
  - `pointer`: Pointer to the memory block to be freed.

## Other Helper Functions

- `get_align(size: usize) -> usize`: Returns the aligned size for the memory block.
- `get_block_head(ptr: *mut ()) -> *mut Header`: Returns a pointer to the header of the memory block.
- `init_malloc() -> Result<(), *mut ()>`: Initializes the memory allocation manager.
- `add_list(size: usize) -> Result<*mut Header, *mut ()>`: Adds a list of memory blocks of size `size`.
- `find_free_mem_block(size: usize) -> Result<*mut Header, *mut ()>`: Finds a free memory block of size `size`.

## Important Notes

- The code uses the `libc` library for low-level memory allocation functions such as `sbrk`, `mmap`, `munmap`, `memcpy`, `memset`, and `write`.
- The `malloc`, `realloc`, `calloc`, and `free` functions provide an interface for memory allocation and deallocation.
- Memory allocation is done in blocks of pre-defined sizes and maintained in free lists for reuse.
- For memory blocks larger than `MAX_BYTE`, the `mmap` allocation mechanism is used.
- The implementation is thread-unsafe and does not handle memory fragmentation.
- Pointer manipulation is done in an unsafe manner using `unsafe` due to the low-level nature of memory allocation operations.
- This implementation is suitable for environments where standard dynamic memory allocation is not available and there are resource limitations.