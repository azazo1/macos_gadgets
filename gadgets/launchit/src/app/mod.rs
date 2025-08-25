mod state;
mod path_utils;
pub use state::*;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::ui;

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: AppState::new()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create app and run it
        let res = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err);
        }

        Ok(())
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| ui::render(f, &mut self.state))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match self.state.current_screen {
                            Screen::Main => {
                                match key.code {
                                    KeyCode::Char('q') => return Ok(()),
                                    KeyCode::Char('j') | KeyCode::Down => self.state.next_item(),
                                    KeyCode::Char('k') | KeyCode::Up => self.state.previous_item(),
                                    KeyCode::Char('n') | KeyCode::Char('c') => {
                                        self.state.new_agent();
                                    }
                                    KeyCode::Char('l') | KeyCode::Enter => {
                                        if let Some(index) = self.state.selected {
                                            if index < self.state.agents.len() {
                                                self.state.agent_detail_index = index;
                                                self.state.current_screen = Screen::Detail;
                                            }
                                        }
                                    }
                                    KeyCode::Char('r') => self.state.refresh_agents()?,
                                    _ => {}
                                }
                            }
                            Screen::Detail => {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Char('q') => {
                                        self.state.current_screen = Screen::Main;
                                    }
                                    KeyCode::Char('j') | KeyCode::Down => self.state.next_action(),
                                    KeyCode::Char('k') | KeyCode::Up => self.state.previous_action(),
                                    KeyCode::Enter => self.state.execute_action()?,
                                    _ => {}
                                }
                            }
                            Screen::NewAgent => {
                                match key.code {
                                    KeyCode::Esc => {
                                        self.state.current_screen = Screen::Main;
                                        self.state.new_agent_form = Default::default();
                                    }
                                    KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                                        self.state.next_form_field();
                                    }
                                    KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                                        self.state.previous_form_field();
                                    }
                                    KeyCode::Enter => {
                                        match self.state.new_agent_form_field {
                                            NewAgentFormField::Domain => {
                                                self.state.next_form_field();
                                            }
                                            NewAgentFormField::Program => {
                                                self.state.next_form_field();
                                            }
                                            NewAgentFormField::Arguments => {
                                                if self.state.edit_mode {
                                                    self.state.edit_mode = false;
                                                } else {
                                                    self.state.edit_mode = true;
                                                }
                                            }
                                            NewAgentFormField::KeepAlive => {
                                                self.state.new_agent_form.keep_alive = !self.state.new_agent_form.keep_alive;
                                                self.state.next_form_field();
                                            }
                                            NewAgentFormField::StdoutPath => {
                                                self.state.next_form_field();
                                            }
                                            NewAgentFormField::StderrPath => {
                                                self.state.next_form_field();
                                            }
                                            NewAgentFormField::UseInterval => {
                                                self.state.new_agent_form.use_interval = !self.state.new_agent_form.use_interval;
                                                self.state.next_form_field();
                                            }
                                            NewAgentFormField::Interval => {
                                                self.state.next_form_field();
                                            }
                                            NewAgentFormField::CreateButton => {
                                                if self.state.create_new_agent()? {
                                                    self.state.current_screen = Screen::Main;
                                                    self.state.new_agent_form = Default::default();
                                                    self.state.refresh_agents()?;
                                                }
                                            }
                                            NewAgentFormField::CancelButton => {
                                                self.state.current_screen = Screen::Main;
                                                self.state.new_agent_form = Default::default();
                                            }
                                        }
                                    }
                                    key => {
                                        // Pass event handling to the state
                                        if self.state.current_screen == Screen::NewAgent {
                                            let _ = self.state.handle_input_event(Event::Key(KeyEvent::new(key, KeyModifiers::empty())));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.state.on_tick()?;
                last_tick = Instant::now();
            }
        }
    }
}
