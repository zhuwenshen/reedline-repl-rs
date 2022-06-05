# reedline-repl-rs

Library to help you create a fancy [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) for your application based on [nushell](https://github.com/nushell/nushell)'s [reedline](https://github.com/nushell/reedline).

Features:
- Interactive tab-completion
- Fish-style history autosuggestions
- Syntax highlighting 
- (optional) file based command History

Basic example code:

 ```rust
use std::collections::HashMap;
use reedline_repl_rs::{Command, Error, Parameter, Result, Value};
use reedline_repl_rs::{Convert, Repl};

// Add two numbers.
fn add<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
    let first: i32 = args["first"].convert()?;
    let second: i32 = args["second"].convert()?;

    Ok(Some((first + second).to_string()))
}

// Write "Hello"
fn hello<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!("Hello, {}", args["who"])))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_prompt("MyApp")
        .with_description("My very cool app")
        .with_banner("Welcome to MyApp")
        .add_command(
            Command::new("add", add)
                .with_parameter(Parameter::new("first").set_required(true)?)?
                .with_parameter(Parameter::new("second").set_required(true)?)?
                .with_help("Add two numbers together"),
        )
        .add_command(
             Command::new("hello", hello)
                 .with_parameter(Parameter::new("who").set_required(true)?)?
                 .with_help("Greetings!"),
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