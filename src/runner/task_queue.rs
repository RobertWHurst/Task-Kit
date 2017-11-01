use std::sync::{Arc, RwLock};
use super::Executable;

#[derive(Debug, Default, Clone)]
pub struct TaskQueue(Arc<RwLock<Vec<Box<Executable>>>>);

impl TaskQueue {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn append(&self, tasks: &mut Vec<Box<Executable>>) {
    self.0.write().unwrap().append(tasks);
  }

  pub fn insert(&self, task: Box<Executable>) {
    self.0.write().unwrap().push(task);
  }

  pub fn split(&self) -> Vec<Box<Executable>> {
    let midpoint = {
      let queue = self.0.read().unwrap();
      if queue.len() < 2 {
        return Vec::new();
      }
      queue.len() / 2
    };
    self.0.write().unwrap().split_off(midpoint)
  }

  pub fn next(&self) -> Option<Box<Executable>> {
    let mut queue = self.0.write().unwrap();
    if queue.len() > 0 {
      Some(queue.remove(0))
    } else {
      None
    }
  }

  pub fn len(&self) -> usize {
    self.0.read().unwrap().len()
  }
}
