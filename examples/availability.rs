use boolcodec::BoolVec;
use std::mem::{size_of, size_of_val};

fn main() {
    const SAMPLES: usize = 48_000;
    let pattern = [true, true, false];

    let mut bool_vec = BoolVec::new();
    let mut vec = Vec::new();
    for bit in pattern.into_iter().cycle().take(SAMPLES) {
        bool_vec.push(bit);
        vec.push(bit);
    }

    let vec_inline = size_of_val(&vec);
    let vec_heap = vec.capacity() * size_of::<bool>();

    let bool_vec_inline = size_of::<BoolVec>();
    let (encoded, logical_len) = bool_vec.into_encoded_parts();
    let bool_vec_heap = encoded.capacity() * size_of::<u8>();
    let vec_total = vec_inline + vec_heap;
    let bool_vec_total = bool_vec_inline + bool_vec_heap;

    println!("availability samples: {logical_len}");
    println!(
        "Vec<bool>:  {vec_inline} inline + {vec_heap} heap = {} bytes",
        vec_total
    );
    println!(
        "BoolVec:    {bool_vec_inline} inline + {bool_vec_heap} heap = {} bytes",
        bool_vec_total
    );
    println!("encoded payload: {} bytes", encoded.len());
    println!(
        "summary: {:.1}x less total memory ({:.1}% smaller)",
        vec_total as f64 / bool_vec_total as f64,
        (1.0 - bool_vec_total as f64 / vec_total as f64) * 100.0
    );
    println!(
        "payload only: {:.1}x smaller",
        vec.len() as f64 / encoded.len() as f64
    );
}
