//! reedline-repl-rs - [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) library
//! for Rust
//!
//! # Example
//!
//! ```
//! use clap::{Arg, ArgMatches, Command};
//! use reedline_repl_rs::{Result, Repl};
//!
//! // Write "Hello"
//! fn hello<T>(args: &ArgMatches, _context: &mut T) -> Result<Option<String>> {
//!     Ok(Some(format!("Hello, {}", args.value_of("who").unwrap())))
//! }
//!
//! fn main() -> Result<()> {
//!     let mut repl = Repl::new(())
//!         .with_name("MyApp")
//!         .with_version("v0.1.0")
//!         .with_description("My very cool app")
//!         .add_command(
//!              Command::new("hello")
//!                  .arg(Arg::new("who").required(true))
//!                  .about("Greetings!"),
//!              hello
//!     );
//!     repl.run()
//!  }
//! ```
//! reedline-repl-rs uses the [builder](https://en.wikipedia.org/wiki/Builder_pattern) pattern extensively.
//! What these lines are doing is:
//! - creating a repl with an empty Context (see below)
//! - with a name of "MyApp", the given version, and the given description
//! - and adding a "hello" command which calls out to the `hello` callback function defined above
//! - the `hello` command has a single parameter, "who", which is required, and has the given help
//! message
//!
//! The `hello` function takes a reference to [ArgMatches](https://docs.rs/clap/latest/clap/struct.ArgMatches.html),
//! and an (unused) `Context`, which is used to hold state if you
//! need to - the initial context is passed in to the call to
//! [Repl::new](struct.Repl.html#method.new), in our case, `()`.
//! Because we're not using a Context, we need to include a generic type in our `hello` function,
//! because there's no way to pass an argument of type `()` otherwise.
//!
//! All command function callbacks return a `Result<Option<String>>`. This has the following
//! effect:
//! - If the return is `Ok(Some(String))`, it prints the string to stdout
//! - If the return is `Ok(None)`, it prints nothing
//! - If the return is an error, it prints the error message to stderr
//!
//! # Context
//!
//! The `Context` type is used to keep state between REPL calls. Here's an example:
//! ```
//! use clap::{ArgMatches, Command};
//! use reedline_repl_rs::{Result, Repl};
//! use std::collections::VecDeque;
//!
//! #[derive(Default)]
//! struct Context {
//!     list: VecDeque<String>,
//! }
//!
//! // Append name to list
//! fn append(args: &ArgMatches, context: &mut Context) -> Result<Option<String>> {
//!     let name: String = matches.value_of("name").unwrap().to_string();
//!     context.list.push_back(name);
//!     let list: Vec<String> = context.list.clone().into();
//!
//!     Ok(Some(list.join(", ")))
//! }
//!
//! // Prepend name to list
//! fn prepend(args: &ArgMatches, context: &mut Context) -> Result<Option<String>> {
//!     let name: String = matches.value_of("name").unwrap().to_string();
//!     context.list.push_front(name);
//!     let list: Vec<String> = context.list.clone().into();
//!
//!     Ok(Some(list.join(", ")))
//! }
//!
//! fn main() -> Result<()> {
//!     let mut repl = Repl::new(Context::default())
//!         .add_command(
//!             Command::new("append")
//!                 .with_parameter(Parameter::new("name").set_required(true)?)?
//!                 .with_help("Append name to end of list"),
//!             append
//!         )
//!         .add_command(
//!             Command::new("prepend")
//!                 .with_parameter(Parameter::new("name").set_required(true)?)?
//!                 .with_help("Prepend name to front of list"),
//!             prepend
//!         );
//!     repl.run()
//! }
//! ```
//! A few things to note:
//! - you pass in the initial value for your Context struct to the call to
//! [Repl::new()](struct.Repl.html#method.new)
//! - the context is passed to your command callback functions as a mutable reference
//!
//! # Help
//! reedline-repl-rs automatically support for supplying help commands for your REPL via clap.
//! ```bash
//! % myapp
//! Welcome to MyApp v0.1.0
//! MyApp> help
//! MyApp v0.1.0: My very cool app
//! ------------------------------
//! append - Append name to end of list
//! prepend - Prepend name to front of list
//! MyApp> help append
//! append: Append name to end of list
//! Usage:
//!         append name
//! MyApp>
//! ```
//!
//! # Errors
//!
//! Your command functions don't need to return `reedline_repl_rs::Error`; you can return any error from
//! them. Your error will need to implement `std::fmt::Display`, so the Repl can print the error,
//! and you'll need to implement `std::convert::From` for `reedline_repl_rs::Error` to your error type.
//! This makes error handling in your command functions easier, since you can just allow whatever
//! errors your functions emit bubble up.
//!
//! ```
//! use reedline_repl_rs::Repl;
//! use clap::{ArgMatches, Command};
//! use std::fmt;
//! use std::result::Result;
//!
//! // My custom error type
//! #[derive(Debug)]
//! enum Error {
//!     DivideByZeroError,
//!     ReplError(reedline_repl_rs::Error),
//! }
//!
//! // Implement conversion from reedline_repl_rs::Error to my error type
//! impl From<reedline_repl_rs::Error> for Error {
//!     fn from(error: reedline_repl_rs::Error) -> Self {
//!         Error::ReplError(error)
//!     }
//! }
//!
//! // My error has to implement Display as well
//! impl fmt::Display for Error {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
//!         match self {
//!             Error::DivideByZeroError => write!(f, "Whoops, divided by zero!"),
//!             Error::ReplError(error) => write!(f, "{}", error),
//!         }
//!     }
//! }
//!
//! // Divide two numbers.
//! fn divide<T>(args: &ArgMatches, _context: &mut T) -> Result<Option<String>, Error> {
//!     let numerator: f32 = matches.value_of("numerator").unwrap().parse().unwrap();
//!     let denominator: f32 = matches.value_of("denominator").unwrap().parse().unwrap();
//!
//!     if denominator == 0.0 {
//!         return Err(Error::DivideByZeroError);
//!     }
//!
//!     Ok(Some((numerator / denominator).to_string()))
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let mut repl = Repl::new(())
//!         .with_name("MyApp")
//!         .with_version("v0.1.0")
//!         .with_description("My very cool app")
//!         .add_command(
//!             Command::new("divide")
//!                 .arg(Arg::new("numerator").required(true))
//!                 .arg(Arg::new("denominator").required(true))
//!                 .about("Divide two numbers"),
//!             divide
//!     );
//!     Ok(repl.run()?)
//! }
//! ```
//!

mod command;
mod completer;
mod error;
mod prompt;
mod repl;

pub use clap;
pub use command::Command;
pub use error::{Error, Result};
#[doc(inline)]
pub use repl::Repl;

use clap::ArgMatches;

/// Command callback function signature
pub type Callback<Context, Error> =
    fn(&ArgMatches, &mut Context) -> std::result::Result<Option<String>, Error>;

/// Initialize the name, version and description of the Repl from your crate name, version and
/// description
#[macro_export]
macro_rules! initialize_repl {
    ($context: expr) => {{
        let repl = Repl::new($context)
            .with_name(crate_name!())
            .with_version(crate_version!())
            .with_description(crate_description!());

        repl
    }};
}
