use helios_sci::mem::allocator::TlsfAllocator;

#[global_allocator]
static ALLOCATOR: TlsfAllocator = TlsfAllocator::new();

fn main() {
    let v = Box::new(42);
    assert_eq!(*v, 42);
    println!("Allocated a box with value: {}", *v);
}
