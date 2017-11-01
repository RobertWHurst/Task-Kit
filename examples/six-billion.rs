extern crate task;

use std::time::Instant;
use task::prelude::*;

fn main() {
  let tasks: Vec<_> = (0..2000000)
    .map(|i| {
      count_to_1000(i).join(count_to_2000(i)).map(|(a, b)| a * b)
      // .finally(move |_| println!("{}", i))
    })
    .collect();

  let mut runner = Runner::new();
  println!("Begin");
  let start = Instant::now();
  for task in tasks {
    runner.run(task);
  }
  runner.finish();
  let duration = start.elapsed();
  println!("End, {:?}", duration);
}

fn count_to_1000<'a>(id: u32) -> Task<'a, u32, ()> {
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

fn count_to_2000<'a>(id: u32) -> Task<'a, u32, ()> {
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
