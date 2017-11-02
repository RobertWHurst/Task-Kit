extern crate num_cpus;
extern crate rand;

pub mod runner;
pub mod task;

pub mod prelude {
  pub use runner::Executable;
  pub use runner::Runner;
  pub use task::State;
  pub use task::State::*;
  pub use task::Task;
}
