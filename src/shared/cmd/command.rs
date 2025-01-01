#[derive(Clone, Debug)]
pub enum CommandType {
    Join,
    Leave,
    Name,
}

#[derive(Clone, Debug)]
pub struct Command {
    // The trigger is the first thing that is typed for the user
    trigger: String,

    // Type of command
    command_type: Option<CommandType>,

    // For explaining the command
    description: String,
    usage: String,
}

impl Command {
    pub fn new(trigger: &str) -> Self {
        let cmd_type: Option<CommandType> = match trigger {
            "/join" => Some(CommandType::Join),
            "/leave" => Some(CommandType::Leave),
            "/name" => Some(CommandType::Name),
            _ => None,
        };
        Self {
            trigger: trigger.to_owned(),
            description: String::new(),
            usage: String::new(),
            command_type: cmd_type,
        }
    }

    pub fn description(mut self, des: &str) -> Self {
        self.description = des.to_owned();
        self
    }

    pub fn usage(mut self, usage: &str) -> Self {
        self.usage = usage.to_owned();
        self
    }

    pub fn get_trigger(&self) -> String {
        self.trigger.clone()
    }

    pub fn get_type(&self) -> Option<&CommandType> {
        if let Some(cmd) = &self.command_type {
            return Some(&cmd);
        }
        return None;
    }
}
