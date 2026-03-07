use helios_sci::allocate::Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new(helios_sci::allocate::AllocateStrategy::NextAvailable);

fn main() {
    let value = Box::new(41_u32);
    assert_eq!(*value, 41);
}
