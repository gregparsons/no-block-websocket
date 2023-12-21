//! command.rs
//!
//! send this to either the client or server as a crossbeam inter-thread message

#[derive(Debug)]
pub enum Cmd {
    Shutdown,
    StartPing,
}