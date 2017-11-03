use std::fmt::{self, Debug};
use std::ops::FnMut;
use std::thread;
use super::State;
use super::super::runner::Executable;

/// Tasks can be used to execute code in Task Kit's runner thread pool.
/// This is the key primive of this crate. It can be used to build and
/// organize asyncronous code paths.
///
/// For more on how to use tasks see the [crate documentation](../index.html).
///
/// #Examples
///
/// Polling example:
///
/// ```
/// # use task_kit::prelude::*;
/// # let mut runner = Runner::new();
/// let mut i = 0;
/// let task: Task<u32, ()> = Task::new(move || {
///   if i < 100 {
///     i += 1;
///     Pending
///   } else {
///     Resolve(i)
///   }
/// });
/// # runner.run(task);
/// # runner.finish();
/// ```
///
/// Long running example:
///
/// ```
/// # use task_kit::prelude::*;
/// # let mut runner = Runner::new();
/// let task: Task<u32, ()> = Task::with(move || {
///   let mut i = 0;
///   while i < 100 {
///     i += 1;
///   }
///   i
/// });
/// # runner.run(task);
/// # runner.finish();
/// ```
pub struct Task<'a, T = (), E = ()> {
  task: Box<FnMut() -> State<T, E> + 'a>,
  state: State<T, E>,
  finally: Option<Box<FnMut(T) + 'a>>,
  catch: Option<Box<FnMut(E) + 'a>>,
}

impl<'a, T, E> Task<'a, T, E>
where
  T: 'a,
  E: 'a,
{
  /// Create a new task from a closure returning a `State`
  ///
  /// Provide the new function a closure that contains the logic you wish
  /// to execute asyncronously. This closure will be executed upon the thread
  /// pool within the runner until your closure returns an instance of
  /// `State::Resolve` containing a value or an instance of `State::Reject`
  /// containing an error value.
  ///
  /// # Arguments
  ///
  /// * `task` - A closure containing code to be executed asyncronously by the
  ///            runner.
  ///
  /// # Examples
  ///
  /// ```
  /// # use task_kit::prelude::*;
  /// # fn do_something_blocking() -> String { String::new() }
  /// # let mut runner = Runner::new();
  /// let task: Task<String, ()> = Task::new(|| Resolve(do_something_blocking()));
  /// # runner.run(task);
  /// # runner.finish();
  /// ```
  pub fn new<F>(task: F) -> Self
  where
    F: FnMut() -> State<T, E> + 'a,
  {
    Self {
      task: Box::new(task),
      state: State::Pending,
      finally: None,
      catch: None,
    }
  }

  /// Create a new task from a value.
  ///
  /// Useful only in cases where you need to pass a task to something, but
  /// already have the value you wish to resolve.
  ///
  /// # Arguments
  ///
  /// * `val` - The value you'd like the task to resolve
  ///
  /// # Examples
  ///
  /// ```
  /// # use task_kit::prelude::*;
  /// # let my_string = String::new();
  /// # let mut runner = Runner::new();
  /// let task: Task<String, ()> = Task::from(my_string);
  /// # runner.run(task);
  /// # runner.finish();
  /// ```
  pub fn from(val: T) -> Self {
    let mut val = Some(val);
    Self {
      task: Box::new(move || match val.take() {
        Some(v) => State::Resolve(v),
        None => State::Resolved,
      }),
      state: State::Pending,
      finally: None,
      catch: None,
    }
  }

  /// Create a new task from a closure returning a value.
  ///
  /// The closure will only be executed once by the runner, and is expected to
  /// return the value you wish to resolve.
  ///
  /// # Arguments
  ///
  /// * `with` - A closure that will return the value you'd like the task to
  ///            resolve.
  ///
  /// # Examples
  ///
  /// ```
  /// # use task_kit::prelude::*;
  /// # fn do_something_blocking() -> String { String::new() }
  /// # let mut runner = Runner::new();
  /// let task: Task<String, ()> = Task::with(|| do_something_blocking());
  /// # runner.run(task);
  /// # runner.finish();
  /// ```
  pub fn with<F>(mut with: F) -> Self
  where
    F: FnMut() -> T + 'a,
  {
    Self {
      task: Box::new(move || State::Resolve(with())),
      state: State::Pending,
      finally: None,
      catch: None,
    }
  }

  /// Create a new merged task from the current task instance and a second task
  ///
  /// Join will return a new task that will resolve a tuple containing the
  /// results from both the task `join` is called upon, and the task passed in.
  ///
  /// Both the current task and the second task passed in will still execute in
  /// parallel.
  ///
  /// # Arguments
  ///
  /// * `task` - A second task to join with the current task
  ///
  /// # Examples
  ///
  /// ```
  /// # use task_kit::prelude::*;
  /// # let mut runner = Runner::new();
  /// # let my_task: Task<String, ()> = Task::from(String::new());
  /// # let my_other_task: Task<String, ()> = Task::from(String::new());
  /// let merged_task = my_task.join(my_other_task);
  /// # runner.run(merged_task);
  /// # runner.finish();
  /// ```
  pub fn join<U>(mut self, mut task: Task<'a, U, E>) -> Task<'a, (T, U), E>
  where
    U: 'a,
  {
    self.finally = None;
    self.catch = None;
    task.finally = None;
    task.catch = None;

    Task::new(move || {
      if self.state.is_pending() {
        self.exec();
      }
      if task.state.is_pending() {
        task.exec();
      }

      if self.state.is_reject() {
        return State::Reject(self.state.take().reject().unwrap());
      }
      if task.state.is_reject() {
        return State::Reject(task.state.take().reject().unwrap());
      }

      if self.state.is_resolve() && task.state.is_resolve() {
        let a_val = self.state.take().resolve().unwrap();
        let b_val = task.state.take().resolve().unwrap();
        return State::Resolve((a_val, b_val));
      }
      State::Pending
    })
  }

  /// Get the task state
  ///
  /// Returns a reference to the internal state of the task
  pub fn state(&self) -> &State<T, E> {
    &self.state
  }

  /// Executes the closure within the task once
  ///
  /// If the task resolves or rejects then the returned option will contain
  /// a result object.
  pub fn poll(&mut self) -> Option<Result<T, E>> {
    self.exec();
    if self.state.is_pending() {
      None
    } else {
      self.state.take().into_result()
    }
  }

  /// Executes the closure within the task blocking until the task completes
  pub fn wait(mut self) -> Option<Result<T, E>> {
    loop {
      self.exec();
      if !self.state.is_pending() {
        break self.state.take().into_result();
      }
    }
  }

  pub fn map<F, U>(self, mut map: F) -> Task<'a, U, E>
  where
    F: FnMut(T) -> U + 'a,
    U: 'a,
  {
    self.then(move |v| State::Resolve(map(v)))
  }

  pub fn then<F, U>(mut self, mut task: F) -> Task<'a, U, E>
  where
    F: FnMut(T) -> State<U, E> + 'a,
    U: 'a,
  {
    Task::new(move || {
      self.exec();

      if self.state.is_resolve() || self.state.is_reject() {
        match self.state.take().into_result().unwrap() {
          Ok(r) => task(r),
          Err(e) => State::Reject(e),
        }
      } else {
        State::Pending
      }
    })
  }

  pub fn finally<F>(mut self, finally: F) -> Self
  where
    F: FnMut(T) + 'a,
  {
    self.finally = Some(Box::new(finally));
    self
  }

  pub fn catch<F>(mut self, catch: F) -> Self
  where
    F: FnMut(E) + 'a,
  {
    self.catch = Some(Box::new(catch));
    self
  }
}

impl<'a, T, E> Debug for Task<'a, T, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Task {{ state: {:?} }}", self.state)
  }
}

impl<'a, T, E> Executable for Task<'a, T, E> {
  fn exec(&mut self) -> bool {
    if !self.state.is_pending() {
      return true;
    }

    self.state = (self.task)();

    if let Some(ref mut finally) = self.finally {
      if self.state.is_resolve() {
        finally(self.state.take().resolve().unwrap());
      }
    } else if let Some(ref mut catch) = self.catch {
      if self.state.is_reject() {
        catch(self.state.take().reject().unwrap());
      }
    }

    !self.state.is_pending()
  }
}

impl<'a, T, E> Task<'a, T, E>
where
  T: PartialEq + 'a,
  E: PartialEq + 'a,
{
  pub fn eq(self, task: Task<'a, T, E>) -> Task<'a, bool, E> {
    self.join(task).map(|(a, b)| a == b)
  }
  pub fn ne(self, task: Task<'a, T, E>) -> Task<'a, bool, E> {
    self.join(task).map(|(a, b)| a != b)
  }
}

impl<'a, T, E> Task<'a, T, E>
where
  T: PartialOrd + 'a,
  E: PartialOrd + 'a,
{
  pub fn lt(self, task: Task<'a, T, E>) -> Task<'a, bool, E> {
    self.join(task).map(|(a, b)| a < b)
  }
  pub fn le(self, task: Task<'a, T, E>) -> Task<'a, bool, E> {
    self.join(task).map(|(a, b)| a <= b)
  }
  pub fn gt(self, task: Task<'a, T, E>) -> Task<'a, bool, E> {
    self.join(task).map(|(a, b)| a > b)
  }
  pub fn ge(self, task: Task<'a, T, E>) -> Task<'a, bool, E> {
    self.join(task).map(|(a, b)| a >= b)
  }
}

unsafe impl<'a, T, E> Send for Task<'a, T, E> {}
unsafe impl<'a, T, E> Sync for Task<'a, T, E> {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn can_create_task() {
    let _: Task<(), ()> = Task::new(|| State::Pending);
  }

  #[test]
  fn can_poll_for_value() {
    let mut i = 5;
    let mut task: Task<_, ()> = Task::new(|| {
      i += 1;
      if i == 20 {
        return State::Resolve(i);
      }
      State::Pending
    });

    let result = loop {
      if let Some(Ok(r)) = task.poll() {
        break r;
      }
    };

    assert_eq!(result, 20);
  }

  #[test]
  fn can_wait_for_value() {
    let mut i = 5;
    let task: Task<_, ()> = Task::new(|| {
      i += 1;
      if i == 20 {
        return State::Resolve(i);
      }
      State::Pending
    });
    assert_eq!(task.wait().unwrap().unwrap(), 20);
  }

  #[test]
  fn can_chain_tasks() {
    let task: Task<_, ()> = Task::new(|| State::Resolve(1))
      .then(|n| State::Resolve(n + 1))
      .then(|n| State::Resolve(n + 2))
      .then(|n| State::Resolve(n + 3));
    assert_eq!(task.wait().unwrap().unwrap(), 7);
  }

  #[test]
  fn can_use_finally() {
    let task: Task<_, ()> = Task::new(|| State::Resolve(1)).finally(|val| assert_eq!(val, 1));
    task.wait();
  }
}
