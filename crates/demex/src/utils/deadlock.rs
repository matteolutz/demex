use std::{thread, time};

use parking_lot::deadlock;

pub fn start_deadlock_checking_thread() {
    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_secs(5));

        let deadlocks = deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        println!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            println!("Deadlock #{}", i);
            for t in threads {
                println!("Thread Id {:#?}", t.thread_id());
                println!("{:#?}", t.backtrace());
            }
        }
    });
}
