use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{self, Color, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, stdout, Write};

const LOGO: &str = r#"
██████╗ ██╗   ██╗███████╗████████╗██╗ ██████╗██████╗ ███████╗ █████╗  ██████╗██╗  ██╗    
██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║██╔════╝██╔══██╗██╔════╝██╔══██╗██╔════╝██║  ██║    
██████╔╝██║   ██║███████╗   ██║   ██║██║     ██████╔╝█████╗  ███████║██║     ███████║    
██╔══██╗██║   ██║╚════██║   ██║   ██║██║     ██╔══██╗██╔══╝  ██╔══██║██║     ██╔══██║    
██║  ██║╚██████╔╝███████║   ██║   ██║╚██████╗██║  ██║███████╗██║  ██║╚██████╗██║  ██║    
╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝   
"#;

pub struct StartupUI {
    terminal_height: u16,
    ips: Vec<String>,
    selected_ip: usize,
    mode: UIMode,
}

enum UIMode {
    IPSelection,
    Selected(String),
}

impl StartupUI {
    pub fn new() -> io::Result<Self> {
        let (_, terminal_height) = crossterm::terminal::size()?;
        Ok(Self {
            terminal_height,
            ips: vec![
                "192.168.1.1".to_string(),
                "192.168.1.2".to_string(),
                "192.168.1.3".to_string(),
                "10.0.0.1".to_string(),
                "10.0.0.2".to_string(),
            ],
            selected_ip: 0,
            mode: UIMode::IPSelection,
        })
    }

    pub fn render(&self) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(
            stdout,
            Clear(ClearType::All),
            Clear(ClearType::Purge),
            cursor::MoveTo(0, 0)
        )?;

        // Draw logo
        execute!(stdout, SetForegroundColor(Color::DarkYellow))?;
        let logo_lines: Vec<&str> = LOGO.split('\n').collect();
        for (i, line) in logo_lines.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, i as u16))?;
            print!("{}", line);
        }
        execute!(stdout, SetForegroundColor(Color::Reset))?;

        // Draw IP selection
        match &self.mode {
            UIMode::IPSelection => {
                println!("\r\nAvailable IPs (Use ↑↓ to navigate, Enter to select, Esc to quit):");
                for (i, ip) in self.ips.iter().enumerate() {
                    if i == self.selected_ip {
                        execute!(stdout, SetForegroundColor(Color::Green))?;
                        println!("\r> {}", ip);
                    } else {
                        execute!(stdout, SetForegroundColor(Color::Reset))?;
                        println!("\r  {}", ip);
                    }
                }
            }
            UIMode::Selected(ip) => {
                execute!(stdout, SetForegroundColor(Color::Green))?;
                println!("\r\nSelected IP: {}", ip);
                println!("\r\nPress Esc to select a different IP");
            }
        }

        stdout.flush()?;
        Ok(())
    }

    pub fn handle_input(&mut self) -> io::Result<Option<String>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match &self.mode {
                    UIMode::IPSelection => match code {
                        KeyCode::Up => {
                            if self.selected_ip > 0 {
                                self.selected_ip -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_ip < self.ips.len() - 1 {
                                self.selected_ip += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let selected = self.ips[self.selected_ip].clone();
                            self.mode = UIMode::Selected(selected.clone());
                            return Ok(Some(selected));
                        }
                        KeyCode::Esc => {
                            return Ok(None);
                        }
                        _ => {}
                    },
                    UIMode::Selected(_) => {
                        if code == KeyCode::Esc {
                            self.mode = UIMode::IPSelection;
                        }
                    }
                }
                self.render()?;
            }
        }
        Ok(None)
    }
}
