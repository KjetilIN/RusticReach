use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{self, stdout, Write};

pub struct TerminalUI {
    input_buffer: String,
    messages: Vec<String>,
    terminal_height: u16,
}

impl TerminalUI {
    pub fn new() -> io::Result<Self> {
        let (_, terminal_height) = crossterm::terminal::size()?;
        Ok(Self {
            input_buffer: String::new(),
            messages: Vec::new(),
            terminal_height,
        })
    }

    pub fn render(&self) -> io::Result<()> {
        let mut stdout = stdout();

        // Clear the screen and move to top
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        // Display messages
        let display_messages = if self.messages.len() > (self.terminal_height - 2) as usize {
            &self.messages[(self.messages.len() - (self.terminal_height - 2) as usize)..]
        } else {
            &self.messages[..]
        };

        for message in display_messages {
            println!("{}\r", message);
        }

        // Draw input line at the bottom
        execute!(
            stdout,
            cursor::MoveTo(0, self.terminal_height - 1),
            Clear(ClearType::CurrentLine)
        )?;
        print!("> {}", self.input_buffer);
        stdout.flush()?;

        Ok(())
    }

    pub fn add_message(&mut self, message: String) {
        let clean_message = message.trim().to_string();
        if !clean_message.is_empty() {
            self.messages.push(clean_message);
            self.render().unwrap();
        }
    }

    pub fn handle_input(&mut self) -> io::Result<Option<String>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Enter => {
                        let message = self.input_buffer.clone();
                        self.input_buffer.clear();
                        self.render()?;
                        return Ok(Some(message));
                    }
                    KeyCode::Backspace => {
                        self.input_buffer.pop();
                        self.render()?;
                    }
                    KeyCode::Char(c) => {
                        self.input_buffer.push(c);
                        self.render()?;
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }
}
