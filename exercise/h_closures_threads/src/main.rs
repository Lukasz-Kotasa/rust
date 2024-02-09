// Silence some warnings so they don't distract from the exercise.
#![allow(dead_code, unused_imports, unused_variables)]
use crossbeam::channel;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

fn pause_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn main() {
    let counter = Arc::new(Mutex::new(0));

   
    let cloned_counter1 = Arc::clone(&counter);
    let handle_t1 = thread::spawn( move || {
        for _ in 0..100 {
            let mut num = cloned_counter1.lock().unwrap();
            *num += 1;
        }
    });

    let cloned_counter2 = Arc::clone(&counter);
    let handle_t2 = thread::spawn( move || {
        for _ in 0..100 {
            let mut num = cloned_counter2.lock().unwrap();
            *num += 1;
        }
    });


    handle_t1.join().unwrap();
    handle_t2.join().unwrap();
    

    // Challenge: Make two child threads and give them each a receiving end to a channel.  From the
    // main thread loop through several values and print each out and then send it to the channel.
    // On the child threads print out the values you receive. Close the sending side in the main
    // thread by calling `drop(tx)` (assuming you named your sender channel variable `tx`).  Join
    // the child threads.
    println!("Main thread: Global value is: {}",  *counter.lock().unwrap());
}
