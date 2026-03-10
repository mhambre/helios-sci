use helios_sci::mem::allocator::FLAllocator;

#[global_allocator]
static ALLOCATOR: FLAllocator = FLAllocator::new();

fn main() {
    const CHUNK_SIZE: usize = 256 * 1024; // 256 KiB
    const CHUNK_COUNT: usize = 8; // 2 MiB total

    let mut chunks = Vec::with_capacity(CHUNK_COUNT);
    for i in 0..CHUNK_COUNT {
        let mut chunk = Box::new([0_u8; CHUNK_SIZE]);
        for (j, byte) in chunk.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(31).wrapping_add((j % 251) as u8);
        }
        chunks.push(chunk);
    }

    assert_eq!(chunks.len() * CHUNK_SIZE, 2 * 1024 * 1024);

    for (i, chunk) in chunks.iter().enumerate() {
        for (j, byte) in chunk.iter().enumerate() {
            let expected = (i as u8).wrapping_mul(31).wrapping_add((j % 251) as u8);
            assert_eq!(
                *byte, expected,
                "memory validation failed at chunk {i}, offset {j}"
            );
        }
    }
}
