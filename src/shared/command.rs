#[derive(Clone, Debug)]
pub struct Command {
    // The trigger is the first thing that is typed for the user
    trigger: String,

    // For explaining the command
    description: String,
    usage: String,
}

impl Command {
    pub fn new(trigger: &str) -> Self {
        Self {
            trigger: trigger.to_owned(),
            description: String::new(),
            usage: String::new(),
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
}
