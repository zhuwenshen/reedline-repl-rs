use crate::{Command, Parameter};
use reedline::{Completer, Span, Suggestion};
use std::collections::HashMap;

struct CompleterCommand {
    pub(crate) name: String,
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) help_summary: Option<String>,
}

impl CompleterCommand {
    pub fn new<Context, E>(command: &Command<Context, E>) -> Self {
        CompleterCommand {
            name: command.name.clone(),
            parameters: command.parameters.clone(),
            help_summary: command.help_summary.clone(),
        }
    }

    pub fn parameter_values_starting_with(
        &self,
        idx: usize,
        prefix: &str,
        start: usize,
        pos: usize,
    ) -> Vec<Suggestion> {
        if let Some(parameter) = self.parameters.get(idx) {
            parameter
                .allowed_values
                .keys()
                .filter(|value| value.starts_with(prefix))
                .map(|value| Suggestion {
                    value: value.clone(),
                    description: parameter
                        .allowed_values
                        .get(value)
                        .unwrap()
                        .as_ref()
                        .map(|s| s.to_string()),
                    extra: None,
                    span: Span::new(start, pos),
                    // span: Default::default(),
                    append_whitespace: idx < self.parameters.len(),
                })
                .collect()
        } else {
            vec![]
        }
    }
}

pub struct ReplCompleter {
    commands: HashMap<String, CompleterCommand>,
}

impl ReplCompleter {
    pub fn new<Context, E>(repl_commands: &HashMap<String, Command<Context, E>>) -> Self {
        let mut commands = HashMap::new();
        for (name, repl_command) in repl_commands.iter() {
            commands.insert(name.clone(), CompleterCommand::new(repl_command));
        }
        ReplCompleter { commands }
    }

    fn commands_starting_with(&self, prefix: &str, pos: usize) -> Vec<Suggestion> {
        let mut result: Vec<Suggestion> = self
            .commands
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(_, command)| Suggestion {
                value: command.name.clone(),
                description: command.help_summary.clone(),
                extra: None,
                span: Span::new(0, pos),
                // span: Default::default(),
                append_whitespace: !command.parameters.is_empty(),
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
                command.parameter_values_starting_with(splitted.count(), last, start, pos)
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
