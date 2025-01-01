use super::command::Command;

pub struct Commands {
    list: Vec<Command>,
}

impl Commands {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn push_command(&mut self, command: Command) {
        self.list.push(command);
    }

    pub fn retrieve_command(&self, input: String) -> Option<Command> {
        for command in &self.list {
            if input.starts_with(&command.get_trigger()) {
                return Some(command.clone());
            }
        }
        return None;
    }
}
