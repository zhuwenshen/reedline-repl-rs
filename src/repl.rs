use crate::completer::ReplCompleter;
use crate::error::*;
use crate::prompt::SimplePrompt;
use crate::Callback;
use crate::command::Command;
use crossterm::event::{KeyCode, KeyModifiers};
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultHinter, DefaultValidator, Emacs,
    ExampleHighlighter, FileBackedHistory, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use yansi::Paint;

type ErrorHandler<Context, E> = fn(error: E, repl: &Repl<Context, E>) -> Result<()>;

fn default_error_handler<Context, E: Display>(error: E, _repl: &Repl<Context, E>) -> Result<()> {
    eprintln!("{}", error);
    Ok(())
}

/// Main REPL struct
pub struct Repl<Context, E: Display> {
    name: String,
    banner: Option<String>,
    version: String,
    description: String,
    prompt: Box<dyn Display>,
    custom_prompt: bool,
    commands: HashMap<String, Command<Context, E>>,
    history: Option<PathBuf>,
    context: Context,
    error_handler: ErrorHandler<Context, E>,
}

impl<Context, E> Repl<Context, E>
where
    E: Display + From<Error> + std::fmt::Debug,
{
    /// Create a new Repl with the given context's initial value.
    pub fn new(context: Context) -> Self {
        let name = String::new();

        Self {
            name: name.clone(),
            banner: None,
            version: String::new(),
            description: String::new(),
            prompt: Box::new(Paint::green(format!("{}> ", name)).bold()),
            custom_prompt: false,
            commands: HashMap::new(),
            history: None,
            context,
            error_handler: default_error_handler,
        }
    }

    /// Give your Repl a name. This is used in the help summary for the Repl.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        if !self.custom_prompt {
            self.prompt = Box::new(Paint::green(format!("{}> ", name)).bold());
        }

        self
    }

    /// Give your Repl a banner. This is printed at the start of running the Repl.
    pub fn with_banner(mut self, banner: &str) -> Self {
        self.banner = Some(banner.to_string());

        self
    }

    /// Give your Repl a version. This is used in the help summary for the Repl.
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();

        self
    }

    /// Give your Repl a description. This is used in the help summary for the Repl.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();

        self
    }

    /// Give your Repl a file based history saved at history_path
    pub fn with_history(mut self, history_path: PathBuf) -> Self {
        self.history = Some(history_path);

        self
    }

    /// Give your Repl a custom prompt. The default prompt is the Repl name, followed by
    /// a `>`, all in green, followed by a space.
    pub fn with_prompt(mut self, prompt: &'static dyn Display) -> Self {
        self.prompt = Box::new(prompt);
        self.custom_prompt = true;

        self
    }

    /// Pass in a custom error handler. This is really only for testing - the default
    /// error handler simply prints the error to stderr and then returns
    pub fn with_error_handler(mut self, handler: ErrorHandler<Context, E>) -> Self {
        self.error_handler = handler;

        self
    }

    /// Add a command to your REPL
    pub fn add_command(
        mut self,
        command: clap::Command<'static>,
        callback: Callback<Context, E>,
    ) -> Self {
        let name = command.get_name().to_string();
        self.commands
            .insert(name.clone(), Command::new(&name, command, callback));
        self
    }

    fn show_help(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            let mut app = clap::Command::new("app");

            for (_, com) in self.commands.iter() {
                app = app.subcommand(com.clap_command.clone());
            }
            let mut help_bytes: Vec<u8> = Vec::new();
            app.write_help(&mut help_bytes)
                .expect("failed to print help");
            let mut help_string =
                String::from_utf8(help_bytes).expect("Help message was invalid UTF8");
            let marker = "SUBCOMMANDS:";
            if let Some(marker_pos) = help_string.find(marker) {
                help_string = "COMMANDS:".to_string()
                    + &help_string[(marker_pos + marker.len())..help_string.len()];
            }
            let header = format!("{} {}: {}", self.name, self.version, self.description);
            let underline = Paint::new(" ".repeat(header.len())).strikethrough();
            println!("{}", header);
            println!("{}", underline);
            println!("{}", help_string);
        } else if let Some((_, subcommand)) = self
            .commands
            .iter()
            .find(|(name, _)| name.as_str() == args[0])
        {
            subcommand
                .clap_command
                .clone()
                .print_help()
                .expect("failed to print help");
        } else {
            eprintln!("Help not found for command '{}'", args[0]);
        }
        Ok(())
    }

    fn handle_command(&mut self, command: &str, args: &[&str]) -> core::result::Result<(), E> {
        match self.commands.get(command) {
            Some(definition) => {
                let mut argv: Vec<&str> = vec![command];
                argv.extend(args);
                match definition
                    .clap_command
                    .clone()
                    .try_get_matches_from_mut(argv)
                {
                    Ok(matches) => match (definition.callback)(&matches, &mut self.context) {
                        Ok(Some(value)) => println!("{}", value),
                        Ok(None) => (),
                        Err(error) => return Err(error),
                    },
                    Err(err) => {
                        err.print().expect("failed to print");
                    }
                };
            }
            None => {
                if command == "help" {
                    self.show_help(args)?;
                } else {
                    return Err(Error::UnknownCommand(command.to_string()).into());
                }
            }
        }

        Ok(())
    }

    fn process_line(&mut self, line: String) -> core::result::Result<(), E> {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let r = regex::Regex::new(r#"("[^"\n]+"|[\S]+)"#).unwrap();
            let args = r
                .captures_iter(trimmed)
                .map(|a| a[0].to_string().replace('\"', ""))
                .collect::<Vec<String>>();
            let mut args = args.iter().fold(vec![], |mut state, a| {
                state.push(a.as_str());
                state
            });
            let command: String = args.drain(..1).collect();
            self.handle_command(&command, &args)?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        enable_virtual_terminal_processing();
        if let Some(banner) = &self.banner {
            println!("{}", banner);
        }
        let prompt = SimplePrompt::new(&self.prompt.to_string());
        let mut commands: Vec<String> = self
            .commands
            .iter()
            .map(|(_, command)| command.name.clone())
            .collect();
        commands.push("help".to_string());
        let completer = Box::new(ReplCompleter::new(&self.commands));
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::Menu("completion_menu".to_string()),
        );
        let validator = Box::new(DefaultValidator);
        let mut line_editor = Reedline::create()
            .with_edit_mode(Box::new(Emacs::new(keybindings)))
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_hinter(Box::new(
                DefaultHinter::default().with_style(Style::new().italic().fg(Color::LightGray)),
            ))
            .with_highlighter(Box::new(ExampleHighlighter::new(commands.clone())))
            .with_validator(validator)
            .with_partial_completions(true)
            .with_quick_completions(false);

        if let Some(history_path) = &self.history {
            let history = FileBackedHistory::with_file(25, history_path.to_path_buf()).unwrap();
            line_editor = line_editor.with_history(Box::new(history));
        }

        loop {
            let sig = line_editor.read_line(&prompt).unwrap();
            match sig {
                Signal::Success(line) => {
                    if let Err(err) = self.process_line(line) {
                        eprintln!("{}", err);
                    }
                }
                Signal::CtrlC => {}
                Signal::CtrlD => {
                    break;
                }
            }
        }
        disable_virtual_terminal_processing();
        Ok(())
    }
}

#[cfg(windows)]
pub fn enable_virtual_terminal_processing() {
    use winapi_util::console::Console;
    if let Ok(mut term) = Console::stdout() {
        let _guard = term.set_virtual_terminal_processing(true);
    }
    if let Ok(mut term) = Console::stderr() {
        let _guard = term.set_virtual_terminal_processing(true);
    }
}

#[cfg(windows)]
pub fn disable_virtual_terminal_processing() {
    use winapi_util::console::Console;
    if let Ok(mut term) = Console::stdout() {
        let _guard = term.set_virtual_terminal_processing(false);
    }
    if let Ok(mut term) = Console::stderr() {
        let _guard = term.set_virtual_terminal_processing(false);
    }
}

#[cfg(not(windows))]
pub fn enable_virtual_terminal_processing() {
    // no-op
}

#[cfg(not(windows))]
pub fn disable_virtual_terminal_processing() {
    // no-op
}
