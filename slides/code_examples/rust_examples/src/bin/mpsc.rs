use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let (tx, rx) = channel();

    let handle = thread::spawn(move || {
        tx.send("Hello from the thread!").unwrap();
    });

    handle.join().unwrap();

    let message = rx.recv().unwrap();
    println!("Received: {}", message);

    // Multiple producers
    let (tx, rx) = channel();
    for i in 0..10 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(i).unwrap();
        });
    }

    for _ in 0..10 {
        let j = rx.recv().unwrap();
        println!("Received {}", j);
        assert!(0 <= j && j < 10);
    }
}
