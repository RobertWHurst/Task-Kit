use num_cpus;
use super::Executable;
use super::TaskQueueSet;
use super::Worker;

#[derive(Debug)]
/// A thread pool for executing tasks.
///
/// Runner contains a pool of workers, each with it's own thread. As tasks are
/// given to the runner to be executed, the workers execute these tasks.
/// In order to prevent idle workers when there is work to be done, the
/// Runner implement's the work stealling model for parallelism. Idle workers
/// will steal tasks from their siblings in order keep the load distributed,
/// and prevent idle cores.
pub struct Runner {
  task_queue_set: TaskQueueSet,
  workers: Vec<Worker>,
}

impl Runner {
  // Create a new task runner
  pub fn new() -> Self {
    let task_queue_set = TaskQueueSet::new();
    let workers = (0..(num_cpus::get() + 1))
      .map(|_| Worker::new(task_queue_set.clone()))
      .collect();

    Self {
      task_queue_set,
      workers,
    }
  }

  // run a task
  pub fn run<E>(&mut self, task: E)
  where
    E: Executable + 'static,
  {
    self.task_queue_set.push_to_rand_queue(Box::new(task));
  }

  pub fn finish(self) {
    for worker in self.workers {
      worker.finish();
    }
  }
}
