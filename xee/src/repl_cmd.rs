use ahash::HashMap;

use crate::repl::RunContext;

type Execute = Box<dyn Fn(&[&str], &mut RunContext) -> anyhow::Result<()>>;

pub(crate) struct CommandDefinition {
    name: &'static str,
    args: Vec<ArgumentDefinition>,
    execute: Execute,
}

#[derive(Default)]
pub(crate) struct CommandDefinitions {
    definitions: Vec<CommandDefinition>,
    by_name: HashMap<&'static str, usize>,
}

#[derive(Default)]
pub(crate) struct ArgumentDefinition {
    default: Option<&'static str>,
}

impl CommandDefinitions {
    pub(crate) fn new(defitions: Vec<CommandDefinition>) -> Self {
        let mut definitions = Self {
            definitions: Vec::new(),
            by_name: HashMap::default(),
        };
        for definition in defitions {
            definitions.add(definition);
        }
        definitions
    }

    pub(crate) fn add(&mut self, definition: CommandDefinition) {
        let index = self.definitions.len();
        self.by_name.insert(definition.name, index);
        self.definitions.push(definition);
    }

    pub(crate) fn execute(
        &self,
        command: &str,
        run_context: &mut RunContext,
    ) -> anyhow::Result<()> {
        let parts = command.split_whitespace().collect::<Vec<_>>();
        let command_s = parts[0];
        let args = &parts[1..];
        let command = self.get(command_s);
        if let Some(command) = command {
            if args.len() > command.args.len() {
                println!("Too many arguments for command: {}", command_s);
                return Ok(());
            }
            let args = command.preprocess_arguments(args);
            if args.len() < command.args.len() {
                println!("Too few arguments for command: {}", command_s);
                return Ok(());
            }
            (command.execute)(&args, run_context)
        } else {
            println!("Unknown command: {}", command_s);
            Ok(())
        }
    }

    fn get(&self, command: &str) -> Option<&CommandDefinition> {
        self.by_name.get(command).map(|&i| &self.definitions[i])
    }
}

impl CommandDefinition {
    pub(crate) fn new(name: &'static str, args: Vec<ArgumentDefinition>, execute: Execute) -> Self {
        Self {
            name,
            args,
            execute,
        }
    }

    fn preprocess_arguments<'a>(&self, args: &[&'a str]) -> Vec<&'a str> {
        let mut result = Vec::new();
        let mut i = 0;
        for arg in &self.args {
            if i < args.len() {
                result.push(args[i]);
                i += 1;
            } else if let Some(default) = arg.default {
                result.push(default);
            }
        }
        result
    }
}
