use std::sync::{Arc, RwLock};
use rand::{thread_rng, Rng};
use super::Executable;
use super::TaskQueue;

#[derive(Debug, Default, Clone)]
pub struct TaskQueueSet(Arc<RwLock<Vec<TaskQueue>>>);

impl TaskQueueSet {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn new_queue(&self) -> TaskQueue {
    let mut task_queues = self.0.write().unwrap();
    let task_queue = TaskQueue::new();
    task_queues.push(task_queue.clone());
    task_queue
  }

  pub fn steal_from_rand_queue(&self) -> Vec<Box<Executable>> {
    let task_queues = self.0.read().unwrap();
    let mut shuffled_task_queues: Vec<_> = task_queues.iter().collect();
    thread_rng().shuffle(&mut shuffled_task_queues);

    match shuffled_task_queues.into_iter().find(|q| q.len() > 1) {
      Some(task_queue) => task_queue.split(),
      None => Vec::new(),
    }
  }

  pub fn push_to_rand_queue(&self, task: Box<Executable>) {
    let task_queues = self.0.read().unwrap();
    let task_queue = thread_rng()
      .choose(&task_queues)
      .expect("No queues to push to");
    task_queue.insert(task);
  }

  pub fn len(&self) -> usize {
    let task_queues = self.0.read().unwrap();
    task_queues.iter().fold(0, |m, q| m + q.len())
  }
}
