use crate::command::Command;
use reedline::{Completer, Span, Suggestion};
use std::collections::HashMap;

pub struct ReplCompleter {
    commands: HashMap<String, clap::Command<'static>>,
}

impl ReplCompleter {
    pub fn new<Context, E>(repl_commands: &HashMap<String, Command<Context, E>>) -> Self {
        let mut commands = HashMap::new();
        for (name, repl_command) in repl_commands.iter() {
            commands.insert(name.clone(), repl_command.clap_command.clone());
        }
        ReplCompleter { commands }
    }

    pub fn parameter_values_starting_with(
        &self,
        command: &clap::Command<'static>,
        _parameter_idx: usize,
        prefix: &str,
        start: usize,
        pos: usize,
    ) -> Vec<Suggestion> {
        let mut completions = vec![];
        for arg in command.get_arguments() {
            if let Some(possible_values) = arg.get_possible_values() {
                completions.extend(
                    possible_values
                        .iter()
                        .filter(|value| value.get_name().starts_with(prefix))
                        .map(|value| Suggestion {
                            value: value.get_name().to_string(),
                            description: value.get_help().map(|n| n.to_string()),
                            extra: None,
                            span: Span::new(start, pos),
                            append_whitespace: true,
                        }),
                );
            }

            if let Some(long) = arg.get_long() {
                let value = "--".to_string() + long;
                if value.starts_with(prefix) {
                    completions.push(Suggestion {
                        value,
                        description: arg.get_help().map(|n| n.to_string()),
                        extra: None,
                        span: Span::new(start, pos),
                        append_whitespace: true,
                    });
                }
            }

            if let Some(short) = arg.get_short() {
                let value = "-".to_string() + &short.to_string();
                if value.starts_with(prefix) {
                    completions.push(Suggestion {
                        value,
                        description: arg.get_help().map(|n| n.to_string()),
                        extra: None,
                        span: Span::new(start, pos),
                        append_whitespace: true,
                    });
                }
            }
        }
        completions
    }

    fn commands_starting_with(&self, prefix: &str, pos: usize) -> Vec<Suggestion> {
        let mut result: Vec<Suggestion> = self
            .commands
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(_, command)| Suggestion {
                value: command.get_name().to_string(),
                description: command.get_about().map(|n| n.to_string()),
                extra: None,
                span: Span::new(0, pos),
                // span: Default::default(),
                // TODO
                // append_whitespace: !command.parameters.is_empty(),
                append_whitespace: true,
            })
            .collect();

        if "help".starts_with(prefix) {
            result.push(Suggestion {
                value: "help".to_string(),
                description: Some("show help".to_string()),
                extra: None,
                span: Span::new(0, pos),
                append_whitespace: false,
            });
        }

        result
    }
}

impl Completer for ReplCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let mut completions = vec![];
        completions.extend(if line.contains(' ') {
            let mut words = line[0..pos].split(' ');
            let first = words.next().unwrap();
            let mut splitted = words.rev();
            if let Some(command) = self.commands.get(first) {
                let last = splitted.next().unwrap();
                let start = line.len() - last.len();
                self.parameter_values_starting_with(command, splitted.count(), last, start, pos)
            } else {
                vec![]
            }
        } else {
            self.commands_starting_with(line, pos)
        });
        completions.dedup();
        completions
    }
}
