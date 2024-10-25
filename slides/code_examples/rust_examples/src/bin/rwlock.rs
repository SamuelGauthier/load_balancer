use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let data = Arc::new(RwLock::new(0));

    let mut handles = vec![];

    for i in 0..5 {
        let data_w = Arc::clone(&data);

        let handle = thread::spawn(move || {
            let mut num = data_w.write().unwrap();
            *num += 1;
            println!("Thread incremented data to: {}", *num);
        });

        handles.push(handle);

        if i < 3 {
            let data_r = Arc::clone(&data);
            let handle = thread::spawn(move || {
                let num = data_r.read().unwrap();
                println!("Read value: {}", *num);
            });

            handles.push(handle);
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final data value: {}", *data.read().unwrap());
}
