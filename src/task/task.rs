use std::fmt::{self, Debug};
use std::ops::FnMut;
use std::thread;
use super::State;
use super::super::runner::Executable;

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

  pub fn poll(&mut self) -> &mut State<T, E> {
    self.exec();
    &mut self.state
  }

  pub fn wait(mut self) -> Option<Result<T, E>> {
    loop {
      let state = self.poll();
      if !state.is_pending() {
        break state.take().into_result();
      }
      thread::yield_now();
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
      let state = self.poll();

      if state.is_resolve() || state.is_reject() {
        match state.take().into_result().unwrap() {
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
      if let &mut State::Resolve(r) = task.poll() {
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
