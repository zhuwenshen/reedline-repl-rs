# reedline-repl-rs

Library to help you create a fancy [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) for your application based on [nushell](https://github.com/nushell/nushell)'s [reedline](https://github.com/nushell/reedline).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/reedline-repl-rs.svg)](https://crates.io/crates/reedline-repl-rs)
[![Documentation](https://docs.rs/reedline-repl-rs/badge.svg)](https://docs.rs/reedline-repl-rs/latest/)

Features:
- Uses [Clap](https://github.com/clap-rs/clap) to define commands and arguments
- Interactive tab-completion
- Fish-style history autosuggestions
- Syntax highlighting 
- (optional) file based command History

Basic example code:

```rust
use reedline_repl_rs::{Repl, Result};
use clap::{Arg, ArgMatches, Command};

// Write "Hello" with given name
fn hello<T>(args: &ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!("Hello, {}", args.value_of("who").unwrap())))
}

const PROMPT: &str = "MyApp";

fn main() -> Result<()> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_prompt(&PROMPT)
        .with_description("My very cool app")
        .with_banner("Welcome to MyApp")
        .add_command(
            Command::new("hello")
                .arg(Arg::new("who").required(true))
                .about("Greetings!"),
            hello
        );
    repl.run()
}
```

Running the example above:

```plain
Welcome to MyApp
MyApp〉help
MyApp v0.1.0: My very cool app

COMMANDS:
    hello    Greetings!
    help     Print this message or the help of the given subcommand(s)

MyApp〉help hello
hello
Greetings!

USAGE:
    hello <who>

ARGS:
    <who>

OPTIONS:
    -h, --help    Print help information
MyApp〉hello Friend
Hello, Friend
MyApp〉
```

## Thanks

Forked from [repl-rs](https://github.com/jacklund/repl-rs) by [Jacklund](https://github.com/jacklund), 
changed to use [reedline](https://github.com/nushell/reedline) which is an advanced readline clone
and the base of [nushell](https://github.com/nushell/nushell).

