use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use std::thread;

fn main() {
    let counter = Arc::new(AtomicI32::new(0));

    let increment = |counter: Arc<AtomicI32>| {
        for _ in 0..10_000 {
            counter.fetch_add(1, Ordering::SeqCst);
        }
    };

    let counter1 = Arc::clone(&counter);
    let t1 = thread::spawn(move || increment(counter1));

    let counter2 = Arc::clone(&counter);
    let t2 = thread::spawn(move || increment(counter2));

    t1.join().unwrap();
    t2.join().unwrap();

    println!("Counter: {}", counter.load(Ordering::SeqCst));
}
