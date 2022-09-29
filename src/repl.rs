use crate::command::ReplCommand;
use crate::completer::ReplCompleter;
use crate::error::*;
use crate::prompt::ReplPrompt;
use crate::{paint_green_bold, paint_yellow_bold, AfterCommandCallback, Callback};
#[cfg(feature = "async")]
use crate::{AsyncAfterCommandCallback, AsyncCallback};
use clap::Command;
use crossterm::event::{KeyCode, KeyModifiers};
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultHinter, DefaultValidator, Emacs,
    ExampleHighlighter, FileBackedHistory, Keybindings, Reedline, ReedlineEvent, ReedlineMenu,
    Signal,
};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

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
    prompt: ReplPrompt,
    after_command_callback: Option<AfterCommandCallback<Context, E>>,
    #[cfg(feature = "async")]
    after_command_callback_async: Option<AsyncAfterCommandCallback<Context, E>>,
    commands: HashMap<String, ReplCommand<Context, E>>,
    history: Option<PathBuf>,
    history_capacity: Option<usize>,
    context: Context,
    keybindings: Keybindings,
    hinter_style: Style,
    hinter_enabled: bool,
    quick_completions: bool,
    partial_completions: bool,
    stop_on_ctrl_c: bool,
    stop_on_ctrl_d: bool,
    error_handler: ErrorHandler<Context, E>,
    init_commands: Vec<String>, // 初始化的命令
}

impl<Context, E> Repl<Context, E>
where
    E: Display + From<Error> + std::fmt::Debug,
{
    /// Create a new Repl with the given context's initial value.
    pub fn new(context: Context) -> Self {
        let name = String::from("repl");
        let style = Style::new().italic().fg(Color::LightGray);
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::Menu("completion_menu".to_string()),
        );
        let prompt = ReplPrompt::new(&paint_green_bold(&format!("{}> ", name)));

        Self {
            name,
            banner: None,
            version: String::new(),
            description: String::new(),
            commands: HashMap::new(),
            history: None,
            history_capacity: None,
            after_command_callback: None,
            #[cfg(feature = "async")]
            after_command_callback_async: None,
            quick_completions: true,
            partial_completions: false,
            hinter_enabled: true,
            hinter_style: style,
            prompt,
            context,
            keybindings,
            stop_on_ctrl_c: false,
            stop_on_ctrl_d: true,
            error_handler: default_error_handler,
            init_commands: vec![],
        }
    }

    /// Give your Repl a name. This is used in the help summary for the Repl.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self.with_formatted_prompt(name)
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

    /// Give your REPL a callback which is called after every command and may update the prompt
    pub fn with_on_after_command(mut self, callback: AfterCommandCallback<Context, E>) -> Self {
        self.after_command_callback = Some(callback);

        self
    }

    /// Give your REPL a callback which is called after every command and may update the prompt
    #[cfg(feature = "async")]
    pub fn with_on_after_command_async(
        mut self,
        callback: AsyncAfterCommandCallback<Context, E>,
    ) -> Self {
        self.after_command_callback_async = Some(callback);

        self
    }

    /// Give your Repl a file based history saved at history_path
    pub fn with_history(mut self, history_path: PathBuf, capacity: usize) -> Self {
        self.history = Some(history_path);
        self.history_capacity = Some(capacity);

        self
    }

    /// Give your Repl a custom prompt. The default prompt is the Repl name, followed by
    /// a `>`, all in green and bold, followed by a space:
    ///
    /// &Paint::green(format!("{}> ", name)).bold().to_string()
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt.update_prefix(prompt);

        self
    }

    /// Give your Repl a custom prompt while applying green/bold formatting automatically
    ///
    /// &Paint::green(format!("{}> ", name)).bold().to_string()
    pub fn with_formatted_prompt(mut self, prompt: &str) -> Self {
        self.prompt.update_prefix(prompt);

        self
    }

    /// Pass in a custom error handler. This is really only for testing - the default
    /// error handler simply prints the error to stderr and then returns
    pub fn with_error_handler(mut self, handler: ErrorHandler<Context, E>) -> Self {
        self.error_handler = handler;

        self
    }

    /// Turn on/off if REPL run is stopped on CTRG+C (Default: false)
    pub fn with_stop_on_ctrl_c(mut self, stop_on_ctrl_c: bool) -> Self {
        self.stop_on_ctrl_c = stop_on_ctrl_c;

        self
    }

    /// Turn on/off if REPL run is stopped on CTRG+D (Default: true)
    pub fn with_stop_on_ctrl_d(mut self, stop_on_ctrl_d: bool) -> Self {
        self.stop_on_ctrl_d = stop_on_ctrl_d;

        self
    }

    /// Turn on quick completions. These completions will auto-select if the completer
    /// ever narrows down to a single entry.
    pub fn with_quick_completions(mut self, quick_completions: bool) -> Self {
        self.quick_completions = quick_completions;

        self
    }

    /// Turn on partial completions. These completions will fill the buffer with the
    /// smallest common string from all the options
    pub fn with_partial_completions(mut self, partial_completions: bool) -> Self {
        self.partial_completions = partial_completions;

        self
    }

    /// Sets the style for reedline's fish-style history autosuggestions
    ///
    /// Default: `nu_ansi_term::Style::new().italic().fg(nu_ansi_term::Color::LightGray)`
    ///
    pub fn with_hinter_style(mut self, style: Style) -> Self {
        self.hinter_style = style;

        self
    }

    /// Disables reedline's fish-style history autosuggestions
    pub fn with_hinter_disabled(mut self) -> Self {
        self.hinter_enabled = false;

        self
    }

    /// Adds a reedline keybinding
    ///
    /// # Panics
    ///
    /// If `comamnd` is an empty [`ReedlineEvent::UntilFound`]
    pub fn with_keybinding(
        mut self,
        modifier: KeyModifiers,
        key_code: KeyCode,
        command: ReedlineEvent,
    ) -> Self {
        self.keybindings.add_binding(modifier, key_code, command);

        self
    }

    /// Find a keybinding based on the modifier and keycode
    pub fn find_keybinding(
        &self,
        modifier: KeyModifiers,
        key_code: KeyCode,
    ) -> Option<ReedlineEvent> {
        self.keybindings.find_binding(modifier, key_code)
    }

    /// Get assigned keybindings
    pub fn get_keybindings(&self) -> HashMap<(KeyModifiers, KeyCode), ReedlineEvent> {
        // keybindings.get_keybindings() cannot be returned directly because KeyCombination is not visible
        self.keybindings
            .get_keybindings()
            .iter()
            .map(|(key, value)| ((key.modifier, key.key_code), value.clone()))
            .collect()
    }

    /// Remove a keybinding
    ///
    /// Returns `Some(ReedlineEvent)` if the keycombination was previously bound to a particular [`ReedlineEvent`]
    pub fn without_keybinding(mut self, modifier: KeyModifiers, key_code: KeyCode) -> Self {
        self.keybindings.remove_binding(modifier, key_code);

        self
    }

    /// Add a command to your REPL
    pub fn with_command(
        mut self,
        command: Command<'static>,
        callback: Callback<Context, E>,
    ) -> Self {
        let name = command.get_name().to_string();
        self.commands
            .insert(name.clone(), ReplCommand::new(&name, command, callback));
        self
    }

    /// 初始化的命令
    pub fn with_init_commands(mut self, init_commands: &[&str]) -> Self {
        let mut init_commands: Vec<_> = init_commands.iter().map(|d| d.to_string()).collect();
        init_commands.reverse();
        self.init_commands = init_commands;
        self
    }

    /// Add a command to your REPL
    #[cfg(feature = "async")]
    pub fn with_command_async(
        mut self,
        command: Command<'static>,
        callback: AsyncCallback<Context, E>,
    ) -> Self {
        let name = command.get_name().to_string();
        self.commands.insert(
            name.clone(),
            ReplCommand::new_async(&name, command, callback),
        );
        self
    }

    fn show_help(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            let mut app = Command::new("app");

            for (_, com) in self.commands.iter() {
                app = app.subcommand(com.command.clone());
            }
            let mut help_bytes: Vec<u8> = Vec::new();
            app.write_help(&mut help_bytes)
                .expect("failed to print help");
            let mut help_string =
                String::from_utf8(help_bytes).expect("Help message was invalid UTF8");
            let marker = "SUBCOMMANDS:";
            if let Some(marker_pos) = help_string.find(marker) {
                help_string = paint_yellow_bold("COMMANDS:")
                    + &help_string[(marker_pos + marker.len())..help_string.len()];
            }
            let header = format!(
                "{} {}\n{}\n",
                paint_green_bold(&self.name),
                self.version,
                self.description
            );
            println!("{}", header);
            println!("{}", help_string);
        } else if let Some((_, subcommand)) = self
            .commands
            .iter()
            .find(|(name, _)| name.as_str() == args[0])
        {
            subcommand
                .command
                .clone()
                .print_help()
                .expect("failed to print help");
            println!();
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
                match definition.command.clone().try_get_matches_from_mut(argv) {
                    Ok(matches) => match (definition
                        .callback
                        .expect("Must be filled for sync commands"))(
                        matches, &mut self.context
                    ) {
                        Ok(Some(value)) => println!("{}", value),
                        Ok(None) => (),
                        Err(error) => return Err(error),
                    },
                    Err(err) => {
                        err.print().expect("failed to print");
                    }
                };
                self.execute_after_command_callback()?;
            }
            None => {
                if command == "help" {
                    self.show_help(args)?;
                } else if command == "exit" {
                    self.stop_on_ctrl_c = true;
                } else {
                    return Err(Error::UnknownCommand(command.to_string()).into());
                }
            }
        }

        Ok(())
    }

    fn execute_after_command_callback(&mut self) -> core::result::Result<(), E> {
        if let Some(callback) = self.after_command_callback {
            match callback(&mut self.context) {
                Ok(Some(new_prompt)) => {
                    self.prompt.update_prefix(&new_prompt);
                }
                Ok(None) => {}
                Err(err) => {
                    eprintln!("failed to execute after_command_callback {:?}", err);
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    async fn execute_after_command_callback_async(&mut self) -> core::result::Result<(), E> {
        self.execute_after_command_callback()?;
        if let Some(callback) = self.after_command_callback_async {
            match callback(&mut self.context).await {
                Ok(new_prompt) => {
                    if let Some(new_prompt) = new_prompt {
                        self.prompt.update_prefix(&new_prompt);
                    }
                }
                Err(err) => {
                    eprintln!("failed to execute after_command_callback {:?}", err);
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "async")]
    async fn handle_command_async(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> core::result::Result<(), E> {
        match self.commands.get(command) {
            Some(definition) => {
                let mut argv: Vec<&str> = vec![command];
                argv.extend(args);
                match definition.command.clone().try_get_matches_from_mut(argv) {
                    Ok(matches) => match if let Some(async_callback) = definition.async_callback {
                        async_callback(matches, &mut self.context).await
                    } else {
                        definition
                            .callback
                            .expect("Either async or sync callback must be set")(
                            matches,
                            &mut self.context,
                        )
                    } {
                        Ok(Some(value)) => println!("{}", value),
                        Ok(None) => (),
                        Err(error) => return Err(error),
                    },
                    Err(err) => {
                        err.print().expect("failed to print");
                    }
                };
                self.execute_after_command_callback_async().await?;
            }
            None => {
                if command == "help" {
                    self.show_help(args)?;
                } else if command == "exit" {
                    self.stop_on_ctrl_c = true;
                } else {
                    return Err(Error::UnknownCommand(command.to_string()).into());
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> (String, Vec<String>) {
        // let r = regex::Regex::new(r#"("[^"\n]+"|[\S]+)"#).unwrap();
        // let args = r
        //     .captures_iter(line)
        //     .map(|a| a[0].to_string().replace('\"', ""))
        //     .collect::<Vec<String>>();
        let mut args = shlex::split(line).unwrap_or_default();
        println!("new :{:?}", args);
        let command: String = args.drain(..1).collect();
        (command, args)
    }

    fn process_line(&mut self, line: String) -> core::result::Result<(), E> {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let (command, args) = self.parse_line(trimmed);
            let args = args.iter().fold(vec![], |mut state, a| {
                state.push(a.as_str());
                state
            });
            self.handle_command(&command, &args)?;
        }
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn process_line_async(&mut self, line: String) -> core::result::Result<(), E> {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let (command, args) = self.parse_line(trimmed);
            let args = args.iter().fold(vec![], |mut state, a| {
                state.push(a.as_str());
                state
            });
            self.handle_command_async(&command, &args).await?;
        }
        Ok(())
    }

    fn build_line_editor(&mut self) -> Result<Reedline> {
        let mut valid_commands: Vec<String> = self
            .commands
            .iter()
            .map(|(_, command)| command.name.clone())
            .collect();
        valid_commands.push("help".to_string());
        let completer = Box::new(ReplCompleter::new(&self.commands));
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
        let validator = Box::new(DefaultValidator);
        let mut line_editor = Reedline::create()
            .with_edit_mode(Box::new(Emacs::new(self.keybindings.clone())))
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_highlighter(Box::new(ExampleHighlighter::new(valid_commands.clone())))
            .with_validator(validator)
            .with_partial_completions(self.partial_completions)
            .with_quick_completions(self.quick_completions);

        if self.hinter_enabled {
            line_editor = line_editor.with_hinter(Box::new(
                DefaultHinter::default().with_style(self.hinter_style),
            ));
        }

        if let Some(history_path) = &self.history {
            let capacity = self.history_capacity.unwrap();
            let history =
                FileBackedHistory::with_file(capacity, history_path.to_path_buf()).unwrap();
            line_editor = line_editor.with_history(Box::new(history));
        }

        Ok(line_editor)
    }

    /// Execute REPL
    pub fn run(&mut self) -> Result<()> {
        enable_virtual_terminal_processing();
        if let Some(banner) = &self.banner {
            println!("{}", banner);
        }
        let mut line_editor = self.build_line_editor()?;

        loop {
            if let Some(line) = self.init_commands.pop() {
                if let Err(err) = self.process_line(line) {
                    (self.error_handler)(err, self)?;
                }
                if self.stop_on_ctrl_c {
                    break;
                }
                continue;
            }
            let sig = line_editor
                .read_line(&self.prompt)
                .expect("failed to read_line");
            match sig {
                Signal::Success(line) => {
                    if let Err(err) = self.process_line(line) {
                        (self.error_handler)(err, self)?;
                    }
                    if self.stop_on_ctrl_c {
                        break;
                    }
                }
                Signal::CtrlC => {
                    if self.stop_on_ctrl_c {
                        break;
                    }
                }
                Signal::CtrlD => {
                    if self.stop_on_ctrl_d {
                        break;
                    }
                }
            }
        }
        disable_virtual_terminal_processing();
        Ok(())
    }

    /// Execute REPL
    #[cfg(feature = "async")]
    pub async fn run_async(&mut self) -> Result<()> {
        enable_virtual_terminal_processing();
        if let Some(banner) = &self.banner {
            println!("{}", banner);
        }
        let mut line_editor = self.build_line_editor()?;

        loop {
            if let Some(line) = self.init_commands.pop() {
                if let Err(err) = self.process_line_async(line).await {
                    (self.error_handler)(err, self)?;
                }
                if self.stop_on_ctrl_c {
                    break;
                }
                continue;
            }
            let sig = line_editor
                .read_line(&self.prompt)
                .expect("failed to read_line");
            match sig {
                Signal::Success(line) => {
                    if let Err(err) = self.process_line_async(line).await {
                        (self.error_handler)(err, self)?;
                    }
                    if self.stop_on_ctrl_c {
                        break;
                    }
                }
                Signal::CtrlC => {
                    if self.stop_on_ctrl_c {
                        break;
                    }
                }
                Signal::CtrlD => {
                    if self.stop_on_ctrl_d {
                        break;
                    }
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
