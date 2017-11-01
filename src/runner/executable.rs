use std::fmt::Debug;
use std::marker::{Send, Sync};

/// Allows an implementer to be executed by the runner
///
/// Implement this trait if you wish to pass a custom type to the runner
/// for execution.
pub trait Executable: Send + Sync + Debug {
  /// Execute the task or other custom type
  ///
  /// Exec is called by the runner repeatedly until it returns true.
  /// Returning true indicates the the task is complete.
  fn exec(&mut self) -> bool;
}
