use crate::Callback;
use std::fmt;
use clap::Command;

/// Struct to define a command in the REPL

pub(crate) struct ReplCommand<Context, E> {
    pub(crate) name: String,
    pub(crate) command: Command<'static>,
    pub(crate) callback: Callback<Context, E>,
}

impl<Context, E> fmt::Debug for ReplCommand<Context, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

impl<Context, E> PartialEq for ReplCommand<Context, E> {
    fn eq(&self, other: &ReplCommand<Context, E>) -> bool {
        self.name == other.name
    }
}

impl<Context, E> ReplCommand<Context, E> {
    /// Create a new command with the given name and callback function
    pub fn new(
        name: &str,
        command: Command<'static>,
        callback: Callback<Context, E>,
    ) -> Self {
        Self {
            name: name.to_string(),
            command,
            callback,
        }
    }
}
