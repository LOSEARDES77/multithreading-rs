# Multithreading Library Written In Rust

---
A simple multithreading library written in rust.

### Usage
```rust
use multithreading::ThreadPool;

fn main() {
    let pool = ThreadPool::new(/*<number_of_threads_to_use>*/);
    for i in 0..10 {
        pool.execute(move || {
            // Do something
            println!("Task {}", i);
        });
    }
}
```

if you want to use all available cores in your cpu set number_of_threads_to_use to 0.

```rust
use multithreading::ThreadPool;
use num_cpus;

fn main() {
    let pool = ThreadPool::new(0);
    for i in 0..10 {
        pool.execute(move || {
            // Do something
            println!("Task {}", i);
        });
    }
}
```

---
You can also get the result of executing something since it returns a receiver.
### Example
```rust
use multithreading::ThreadPool;

fn main() {
    let pool = ThreadPool::new(4);
    let a: u8 = 1;
    let b: u8 = 3;
    let receiver = pool.execute(move || {
        a + b
    });

    let result = receiver.recv().unwrap(); // The value of this is 4 (3+1). 
    assert_eq!(result, 4);
}

```


