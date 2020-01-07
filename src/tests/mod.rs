use crate::{
    buddy_alloc::{block_size, BuddyAlloc},
    LEAF_SIZE,
};

const HEAP_SIZE: usize = 1024 * 1024;

fn with_allocator<F: FnOnce(BuddyAlloc)>(heap_size: usize, f: F) {
    let buf: Vec<u8> = Vec::with_capacity(heap_size);
    unsafe {
        let allocator = BuddyAlloc::new(buf.as_ptr() as usize, buf.as_ptr() as usize + HEAP_SIZE);
        f(allocator);
    }
}

// find a max k that less than n bytes
pub fn first_down_k(n: usize) -> Option<usize> {
    let mut k: usize = 0;
    let mut size = LEAF_SIZE;
    while size < n {
        k += 1;
        size *= 2;
    }
    if size != n {
        k.checked_sub(1)
    } else {
        Some(k)
    }
}

#[test]
fn test_available_bytes() {
    with_allocator(HEAP_SIZE, |allocator| {
        let available_bytes = allocator.available_bytes();
        assert!(available_bytes > (HEAP_SIZE as f64 * 0.8) as usize);
    });
}

#[test]
fn test_basic_malloc() {
    // alloc a min block
    with_allocator(HEAP_SIZE, |mut allocator| {
        let p = allocator.malloc(512);
        let p_addr = p as usize;
        assert!(!p.is_null());
        // memory writeable
        unsafe { p.write(42) };
        assert_eq!(p_addr, p as usize);
        assert_eq!(unsafe { *p }, 42);
    });
}

#[test]
fn test_multiple_malloc() {
    with_allocator(HEAP_SIZE, |mut allocator| {
        let mut available_bytes = allocator.available_bytes();
        let mut count = 0;
        // alloc serveral sized blocks
        while available_bytes >= LEAF_SIZE {
            let k = first_down_k(available_bytes - 1).unwrap_or_default();
            let bytes = block_size(k);
            assert!(!allocator.malloc(bytes).is_null());
            available_bytes -= bytes;
            count += 1;
        }
        assert_eq!(count, 11);
    });
}

#[test]
fn test_small_size_malloc() {
    with_allocator(HEAP_SIZE, |mut allocator| {
        let mut available_bytes = allocator.available_bytes();
        while available_bytes >= LEAF_SIZE {
            assert!(!allocator.malloc(LEAF_SIZE).is_null());
            available_bytes -= LEAF_SIZE;
        }
        // memory should be drained, we can't allocate even 1 byte
        assert!(allocator.malloc(1).is_null());
    });
}

#[test]
fn test_fail_malloc() {
    // not enough memory since we only have HEAP_SIZE bytes,
    // and the allocator itself occupied few bytes
    with_allocator(HEAP_SIZE, |mut allocator| {
        let p = allocator.malloc(HEAP_SIZE);
        assert!(p.is_null());
    });
}

#[test]
fn test_malloc_and_free() {
    fn _test_malloc_and_free(times: usize, heap_size: usize) {
        with_allocator(heap_size, |mut allocator| {
            for _i in 0..times {
                let mut available_bytes = allocator.available_bytes();
                let mut ptrs = Vec::new();
                // alloc serveral sized blocks
                while available_bytes >= LEAF_SIZE {
                    let k = first_down_k(available_bytes - 1).unwrap_or_default();
                    let bytes = block_size(k);
                    let p = allocator.malloc(bytes);
                    assert!(!p.is_null());
                    ptrs.push(p);
                    available_bytes -= bytes;
                }
                // space is drained
                assert!(allocator.malloc(1).is_null());
                // free allocated blocks
                for ptr in ptrs {
                    allocator.free(ptr);
                }
            }
        });
    }
    // test with heaps: 1M, 2M, 4M, 8M
    for i in &[1, 2, 4, 8] {
        _test_malloc_and_free(10, i * HEAP_SIZE);
    }
}
