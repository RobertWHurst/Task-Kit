use std::fmt::{self, Debug};
use std::mem;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum State<T, E> {
  Pending,
  Resolve(T),
  Resolved,
  Reject(E),
  Rejected,
}

impl<T, E> State<T, E> {
  pub fn is_pending(&self) -> bool {
    if let &State::Pending = self {
      return true;
    }
    false
  }

  pub fn is_resolve(&self) -> bool {
    if let &State::Resolve(_) = self {
      return true;
    }
    false
  }

  pub fn is_resolved(&self) -> bool {
    if let &State::Resolved = self {
      return true;
    }
    false
  }

  pub fn is_reject(&self) -> bool {
    if let &State::Reject(_) = self {
      return true;
    }
    false
  }

  pub fn is_rejected(&self) -> bool {
    if let &State::Rejected = self {
      return true;
    }
    false
  }

  pub fn resolve(self) -> Option<T> {
    if let State::Resolve(r) = self {
      Some(r)
    } else {
      None
    }
  }

  pub fn reject(self) -> Option<E> {
    if let State::Reject(e) = self {
      Some(e)
    } else {
      None
    }
  }

  pub fn as_ref(&self) -> State<&T, &E> {
    match self {
      &State::Pending => State::Pending,
      &State::Resolve(ref r) => State::Resolve(r),
      &State::Resolved => State::Resolved,
      &State::Reject(ref e) => State::Reject(e),
      &State::Rejected => State::Rejected,
    }
  }

  pub fn as_mut(&mut self) -> State<&mut T, &mut E> {
    match self {
      &mut State::Pending => State::Pending,
      &mut State::Resolve(ref mut r) => State::Resolve(r),
      &mut State::Resolved => State::Resolved,
      &mut State::Reject(ref mut e) => State::Reject(e),
      &mut State::Rejected => State::Rejected,
    }
  }

  pub fn into_result(self) -> Option<Result<T, E>> {
    match self {
      State::Pending => None,
      State::Resolve(r) => Some(Ok(r)),
      State::Resolved => None,
      State::Reject(e) => Some(Err(e)),
      State::Rejected => None,
    }
  }

  pub fn map<U, F>(self, op: F) -> State<U, E>
  where
    F: FnOnce(T) -> U,
  {
    match self {
      State::Pending => State::Pending,
      State::Resolve(r) => State::Resolve(op(r)),
      State::Resolved => State::Resolved,
      State::Reject(e) => State::Reject(e),
      State::Rejected => State::Rejected,
    }
  }

  pub fn map_err<F, O>(self, op: F) -> State<T, O>
  where
    F: FnOnce(E) -> O,
  {
    match self {
      State::Pending => State::Pending,
      State::Resolve(r) => State::Resolve(r),
      State::Resolved => State::Resolved,
      State::Reject(e) => State::Reject(op(e)),
      State::Rejected => State::Rejected,
    }
  }

  pub fn and<U>(self, res: State<U, E>) -> State<U, E> {
    if let State::Reject(e) = self {
      return State::Reject(e);
    }
    res
  }

  pub fn and_then<U, F>(self, op: F) -> State<U, E>
  where
    F: FnOnce(T) -> State<U, E>,
  {
    match self {
      State::Pending => State::Pending,
      State::Resolve(r) => op(r),
      State::Resolved => State::Resolved,
      State::Reject(e) => State::Reject(e),
      State::Rejected => State::Rejected,
    }
  }

  pub fn or<O>(self, res: State<T, O>) -> State<T, O> {
    if let State::Resolve(r) = self {
      return State::Resolve(r);
    }
    res
  }

  pub fn or_else<O, F>(self, op: F) -> State<T, O>
  where
    F: FnOnce(E) -> State<T, O>,
  {
    match self {
      State::Pending => State::Pending,
      State::Resolve(r) => State::Resolve(r),
      State::Resolved => State::Resolved,
      State::Reject(e) => op(e),
      State::Rejected => State::Rejected,
    }
  }

  pub fn unwrap_or(self, or: T) -> T {
    if let State::Resolve(r) = self {
      r
    } else {
      or
    }
  }

  pub fn unwrap_or_else<F>(self, f: F) -> T
  where
    F: FnOnce() -> T,
  {
    if let State::Resolve(r) = self {
      r
    } else {
      (f)()
    }
  }

  pub fn take(&mut self) -> State<T, E> {
    match self {
      &mut State::Pending => State::Pending,
      &mut State::Resolve(_) => mem::replace(self, State::Resolved),
      &mut State::Resolved => State::Resolved,
      &mut State::Reject(_) => mem::replace(self, State::Rejected),
      &mut State::Rejected => State::Rejected,
    }
  }
}

impl<T, E> State<T, E>
where
  E: fmt::Debug,
{
  pub fn unwrap(self) -> T {
    match self {
      State::Pending => panic!("called `State::unwrap()` on a `State::Pending` value"),
      State::Resolve(r) => r,
      State::Resolved => panic!("called `State::unwrap()` on a `State::Resolved` value"),
      State::Reject(e) => panic!(
        "called `State::unwrap()` on a `State::Reject` value: {:?}",
        e
      ),
      State::Rejected => panic!("called `State::unwrap()` on a `State::Rejected` value"),
    }
  }

  pub fn expect(self, msg: &str) -> T {
    if let State::Resolve(r) = self {
      return r;
    } else if let State::Reject(e) = self {
      panic!("{}: {:?}", msg, e);
    }
    panic!("{}", msg);
  }
}

impl<T, E> State<T, E>
where
  T: fmt::Debug,
{
  pub fn unwrap_reject(self) -> E {
    match self {
      State::Pending => panic!("called `State::unwrap_reject()` on a `State::Pending` value"),
      State::Resolve(r) => panic!(
        "called `State::unwrap_reject()` on a `State::Resolve` value: {:?}",
        r
      ),
      State::Resolved => panic!("called `State::unwrap_reject()` on a `State::Resolved` value"),
      State::Reject(e) => e,
      State::Rejected => panic!("called `State::unwrap_reject()` on a `State::Rejected` value"),
    }
  }

  pub fn expect_reject(self, msg: &str) -> E {
    if let State::Reject(e) = self {
      return e;
    } else if let State::Resolve(r) = self {
      panic!("{}: {:?}", msg, r);
    }
    panic!("{}", msg);
  }
}

impl<T, E> State<T, E>
where
  T: Default,
{
  pub fn unwrap_or_default(self) -> T {
    if let State::Resolve(r) = self {
      r
    } else {
      T::default()
    }
  }
}

impl<T, E> State<T, E>
where
  T: Clone,
  E: Clone,
{
  pub fn clone(self) -> Self {
    match self {
      State::Pending => State::Pending,
      State::Resolve(r) => State::Resolve(r.clone()),
      State::Resolved => State::Resolved,
      State::Reject(e) => State::Reject(e.clone()),
      State::Rejected => State::Rejected,
    }
  }

  pub fn clone_from(&mut self, source: &Self) {
    *self = match source {
      &State::Pending => State::Pending,
      &State::Resolve(ref r) => State::Resolve(r.clone()),
      &State::Resolved => State::Resolved,
      &State::Reject(ref e) => State::Reject(e.clone()),
      &State::Rejected => State::Rejected,
    };
  }
}

impl<T, E> Debug for State<T, E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &State::Pending => write!(f, "State::Pending"),
      &State::Resolve(_) => write!(f, "State::Resolve"),
      &State::Resolved => write!(f, "State::Resolved"),
      &State::Reject(_) => write!(f, "State::Reject"),
      &State::Rejected => write!(f, "State::Rejected"),
    }
  }
}

impl<T, E> From<T> for State<T, E> {
  fn from(val: T) -> Self {
    State::Resolve(val)
  }
}
