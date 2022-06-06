use crate::Callback;
use std::fmt;

/// Struct to define a command in the REPL

pub struct Command<Context, E> {
    pub(crate) name: String,
    pub(crate) clap_command: clap::Command<'static>,
    pub(crate) callback: Callback<Context, E>,
}

impl<Context, E> fmt::Debug for Command<Context, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command").field("name", &self.name).finish()
    }
}

impl<Context, E> std::cmp::PartialEq for Command<Context, E> {
    fn eq(&self, other: &Command<Context, E>) -> bool {
        self.name == other.name
    }
}

impl<Context, E> Command<Context, E> {
    /// Create a new command with the given name and callback function
    pub fn new(
        name: &str,
        clap_command: clap::Command<'static>,
        callback: Callback<Context, E>,
    ) -> Self {
        Self {
            name: name.to_string(),
            clap_command,
            callback,
        }
    }
}
