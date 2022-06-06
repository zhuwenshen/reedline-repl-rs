# reedline-repl-rs

Library to help you create a fancy [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) for your application based on [nushell](https://github.com/nushell/nushell)'s [reedline](https://github.com/nushell/reedline).

Features:
- Uses [Clap](https://github.com/clap-rs/clap) to define commands and arguments
- Interactive tab-completion
- Fish-style history autosuggestions
- Syntax highlighting 
- (optional) file based command History

Basic example code:

 ```rust
use reedline_repl_rs::{Repl, Result, Error};
use clap::{Arg, ArgMatches, Command};

// Add two numbers.
fn add<T>(args: &ArgMatches, _context: &mut T) -> Result<Option<String>> {
    let first: i32 = matches.value_of("first").unwrap().parse().unwrap();
    let second: i32 = matches.value_of("second").unwrap().parse().unwrap();

    Ok(Some((first + second).to_string()))
}

// Write "Hello"
fn hello<T>(args: &ArgMatches, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!("Hello, {}", matches.value_of("first").unwrap())))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_prompt("MyApp")
        .with_description("My very cool app")
        .with_banner("Welcome to MyApp")
        .add_command(
            Command::new("add")
                .arg(Arg::new("first").required(true))
                .arg(Arg::new("second").required(true))
                .about("Add two numbers together"),
            add
        )
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

```bash
% my_app
Welcome to MyApp
MyApp> help
MyApp v0.1.0: My very cool app
------------------------------                              
add - Add two numbers together
hello - Greetings!
MyApp> help add
add: Add two numbers together
Usage:
        add first second
MyApp> add 1 2
3
MyApp> 
```

## Thanks

Forked from [repl-rs](https://github.com/jacklund/repl-rs) by [Jacklund](https://github.com/jacklund), changed to use [reedline](https://github.com/nushell/reedline) which an advanced readline clone and the base of [nushell](https://github.com/nushell/nushell).