use clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

// Write "Hello" with given name
async fn hello<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!("Hello, {}", args.value_of("who").unwrap())))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .add_command_async(
            Command::new("hello")
                .arg(Arg::new("who").required(true))
                .about("Greetings!"),
            |args, context| Box::pin(hello(args, context)),
        );
    repl.run_async().await
}
