extern crate reedline_repl_rs;

use clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::Repl;
use reedline_repl_rs::Result;
use std::collections::VecDeque;

/// Example using Repl with Context

#[derive(Default)]
struct Context {
    list: VecDeque<String>,
}

// Append name to list
fn append(args: &ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let name: String = args.value_of("name").unwrap().to_string();
    context.list.push_back(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

// Prepend name to list
fn prepend(args: &ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let name: String = args.value_of("name").unwrap().to_string();
    context.list.push_front(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(Context::default())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_description("My very cool app")
        .add_command(
            Command::new("append")
                .arg(Arg::new("name").required(true))
                .about("Append name to end of list"),
            append,
        )
        .add_command(
            Command::new("prepend")
                .arg(Arg::new("name").required(true))
                .about("Prepend name to front of list"),
            prepend,
        );
    repl.run()
}
