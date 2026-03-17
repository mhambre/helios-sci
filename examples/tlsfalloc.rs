use helios_sci::mem::allocator::TlsfAllocator;

#[global_allocator]
static ALLOCATOR: TlsfAllocator = TlsfAllocator::new();

use std::collections::VecDeque;

const ITERATIONS: usize = 200_000;
const MAX_LIVE: usize = 10_000;

fn main() {
    println!("Starting allocator stress test");

    {
        let v = Box::new(42);
        assert_eq!(*v, 42);
        println!("Box allocated with value: {}", *v);
    }
    {
        let v = Box::new(47);
        assert_eq!(*v, 47);
        println!("Box allocated with value: {}", *v);
    }

    let mut live = VecDeque::new();

    for i in 0..ITERATIONS {
        let size = (i % 4096) + 1;

        let mut v = Vec::with_capacity(size);
        v.resize(size, (i % 255) as u8);

        for b in &v {
            assert_eq!(*b, (i % 255) as u8);
        }

        live.push_back(v);

        if live.len() > MAX_LIVE {
            live.pop_front();
        }

        if i % 10_000 == 0 {
            println!("iteration {}", i);
        }
    }

    drop(live);

    println!("Allocator stress test completed successfully");
}
