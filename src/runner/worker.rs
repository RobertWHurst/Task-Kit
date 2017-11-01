use std::sync::{Arc, Mutex};
use std::thread::{self, yield_now, JoinHandle};
use super::{Executable, TaskQueue, TaskQueueSet};

#[derive(Debug)]
pub struct Worker {
  is_running: Arc<Mutex<bool>>,
  inner_handle: JoinHandle<()>,
  task_queue: TaskQueue,
}

impl Worker {
  pub fn new(task_queue_set: TaskQueueSet) -> Self {
    let is_running = Arc::new(Mutex::new(true));
    let task_queue = task_queue_set.new_queue();
    let inner_handle = WorkerInner::init(is_running.clone(), task_queue_set, task_queue.clone());
    Self {
      is_running,
      task_queue,
      inner_handle,
    }
  }

  pub fn run<E>(&self, task: E)
  where
    E: Executable + 'static,
  {
    self.task_queue.insert(Box::new(task));
  }

  pub fn finish(self) {
    *self.is_running.lock().unwrap() = false;
    self.inner_handle.join().unwrap();
  }
}

pub struct WorkerInner {
  is_running: Arc<Mutex<bool>>,
  task_queue_set: TaskQueueSet,
  task_queue: TaskQueue,
}

impl WorkerInner {
  fn init(
    is_running: Arc<Mutex<bool>>,
    task_queue_set: TaskQueueSet,
    task_queue: TaskQueue,
  ) -> JoinHandle<()> {
    thread::spawn(move || {
      WorkerInner::new(is_running, task_queue_set, task_queue).run();
    })
  }

  fn new(
    is_running: Arc<Mutex<bool>>,
    task_queue_set: TaskQueueSet,
    task_queue: TaskQueue,
  ) -> Self {
    Self {
      is_running,
      task_queue_set,
      task_queue,
    }
  }

  fn run(&self) {
    loop {
      match self.task_queue.next() {
        Some(mut task) => loop {
          if task.exec() {
            break;
          }
        },
        None => if !self.try_steal() {
          if !*self.is_running.lock().unwrap() {
            break;
          }
          yield_now();
        },
      }
    }
  }

  fn try_steal(&self) -> bool {
    let mut tasks = self.task_queue_set.steal_from_rand_queue();
    if tasks.len() > 0 {
      self.task_queue.append(&mut tasks);
      true
    } else {
      false
    }
  }
}
