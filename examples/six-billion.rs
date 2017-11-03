extern crate task_kit;

use std::time::Instant;
use task_kit::prelude::*;

fn main() {
  let tasks: Vec<_> = (0..20000)
    .map(|i| {
      count_to_1000().join(count_to_2000()).map(|(a, b)| a * b)
      // .finally(move |_| println!("{}", i))
    })
    .collect();

  let mut runner = Runner::new();
  println!("Running...");
  let start = Instant::now();
  runner.run_all(tasks);
  runner.finish();
  let duration = start.elapsed();
  println!("Took {:?} to complete", duration);
}

fn count_to_1000<'a>() -> Task<'a, u32, ()> {
  let mut i = 0;
  Task::new(move || {
    i += 1;
    if i == 1000 {
      State::from(i)
    } else {
      State::Pending
    }
  })
}

fn count_to_2000<'a>() -> Task<'a, u32, ()> {
  let mut i = 0;
  Task::new(move || {
    i += 1;
    if i == 2000 {
      State::Resolve(i)
    } else {
      State::Pending
    }
  })
}
