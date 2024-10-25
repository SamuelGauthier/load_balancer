use std::thread;

static mut COUNTER: i32 = 0;

fn main() {
    let increment = move || {
        for _ in 0..10_000 {
            unsafe {
                // doesn't compile without
                COUNTER += 1;
            }
        }
    };

    let t1 = thread::spawn(increment);
    let t2 = thread::spawn(increment);

    t1.join().unwrap();
    t2.join().unwrap();

    unsafe {
        // doesn't compile without
        println!("Counter: {}", COUNTER);
    }
}
