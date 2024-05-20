# Multithreading Library Written In Rust

---
A simple multithreading library written in rust.

## Usage
```rust
use multithreading::ThreadPool;

fn main() {
    let pool = ThreadPool::new(<number_of_threads_to_use>);
    for i in 0..10 {
        pool.execute(move || {
            // Do something
            println!("Task {}", i);
        });
    }
}
```

if you want to use all available threads, you can use the crate `num_cpus` to get the number of available threads.

```rust
use multithreading::ThreadPool;
use num_cpus;

fn main() {
    let pool = ThreadPool::new(num_cpus::get());
    for i in 0..10 {
        pool.execute(move || {
            // Do something
            println!("Task {}", i);
        });
    }
}
``` 


