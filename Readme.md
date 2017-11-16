__Do not use Task Kit__: This was a personal toy project. Use futures instead => https://github.com/alexcrichton/futures-rs

# Task Kit

__Easy asyncronous tasks for Rust programmers__

The purpose of task kit is to provide a easy to use set of types for building
asyncronous logic. These types include to key pieces, `Task` and `Runner`.

`Task` encapulates polling or long running logic that is desired to be non
blocking. Tasks then can be passed to a `Runner` to be executed.

*Note a `Task` can be executed without the `Runner` as well, but it will be
up to you to deal with control flow and threading.*

- [Latest documentation for Task Kit](https://docs.rs/task_kit)

## Quick Start

Using Task Kit is pretty straight forward. The first thing you should
understand is how to create tasks. We'll start with an example that 
