use anyhow::{Context, Result};
use tui_textarea::{Input, TextArea};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::path_utils;
use crate::launchctl::{create_launch_agent, get_user_agents, LaunchAgent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    Detail,
    NewAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailAction {
    Load,
    Enable,
    Disable,
    Unload,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewAgentFormField {
    Domain,
    Program,
    Arguments,
    KeepAlive,
    StdoutPath,
    StderrPath,
    UseInterval,
    Interval,
    CreateButton,
    CancelButton,
}

#[derive(Debug, Clone)]
pub struct NewAgentForm {
    pub domain_input: TextArea<'static>,
    pub program_input: tui_textarea::Input,
    pub arguments: Vec<String>,
    pub keep_alive: bool,
    pub stdout_path_input: tui_textarea::Input,
    pub stderr_path_input: tui_textarea::Input,
    pub use_interval: bool,
    pub interval: u32,
    pub interval_input: tui_textarea::Input,
    pub argument_input: tui_textarea::Input,
    pub path_suggestions: Vec<String>,
    pub show_suggestions: bool,
    pub selected_suggestion: usize,
    pub input_mode: bool,
}

impl Default for NewAgentForm {
    fn default() -> Self {
        Self {
            domain_input: tui_textarea::Input::default(),
            program_input: tui_textarea::Input::default(),
            arguments: Vec::new(),
            keep_alive: true,
            stdout_path_input: tui_textarea::Input::from("/dev/null"),
            stderr_path_input: tui_textarea::Input::from("/dev/null"),
            use_interval: false,
            interval: 3600,
            interval_input: tui_textarea::Input::from("3600"),
            argument_input: tui_textarea::Input::default(),
            path_suggestions: Vec::new(),
            show_suggestions: false,
            selected_suggestion: 0,
            input_mode: false,
        }
    }
}

pub struct AppState {
    pub current_screen: Screen,
    pub agents: Vec<LaunchAgent>,
    pub selected: Option<usize>,
    pub agent_detail_index: usize,
    pub detail_actions: Vec<DetailAction>,
    pub selected_action: usize,

    // New agent form state
    pub new_agent_form: NewAgentForm,
    pub new_agent_form_field: NewAgentFormField,
    pub new_argument: String,
    pub edit_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Main,
            agents: Vec::new(),
            selected: Some(0),
            agent_detail_index: 0,
            detail_actions: vec![
                DetailAction::Load,
                DetailAction::Enable,
                DetailAction::Disable,
                DetailAction::Unload,
                DetailAction::Back,
            ],
            selected_action: 0,
            new_agent_form: Default::default(),
            new_agent_form_field: NewAgentFormField::Domain,
            new_argument: String::new(),
            edit_mode: false,
        }
    }
}

impl AppState {
    pub fn new() -> Result<Self> {
        let mut state = Self::default();
        state.refresh_agents()?;
        Ok(state)
    }

    pub fn refresh_agents(&mut self) -> Result<()> {
        self.agents = get_user_agents()?;
        Ok(())
    }

    pub fn next_item(&mut self) {
        if !self.new_agent_form.input_mode {
            let len = self.agents.len();
            if len == 0 {
                self.selected = None;
                return;
            }

            if let Some(selected) = self.selected {
                if selected >= len - 1 {
                    self.selected = Some(0);
                } else {
                    self.selected = Some(selected + 1);
                }
            } else {
                self.selected = Some(0);
            }
        }
    }

    pub fn previous_item(&mut self) {
        if !self.new_agent_form.input_mode {
            let len = self.agents.len();
            if len == 0 {
                self.selected = None;
                return;
            }

            if let Some(selected) = self.selected {
                if selected == 0 {
                    self.selected = Some(len - 1);
                } else {
                    self.selected = Some(selected - 1);
                }
            } else {
                self.selected = Some(0);
            }
        }
    }

    pub fn next_action(&mut self) {
        let len = self.detail_actions.len();
        self.selected_action = (self.selected_action + 1) % len;
    }

    pub fn previous_action(&mut self) {
        let len = self.detail_actions.len();
        self.selected_action = (self.selected_action + len - 1) % len;
    }

    pub fn execute_action(&mut self) -> Result<()> {
        if self.agent_detail_index >= self.agents.len() {
            self.current_screen = Screen::Main;
            return Ok(());
        }

        let agent = &self.agents[self.agent_detail_index];
        let action = self.detail_actions[self.selected_action];

        match action {
            DetailAction::Load => {
                let output = std::process::Command::new("launchctl")
                    .args(["bootstrap", &format!("gui/{}", unsafe { libc::getuid() }), agent.path.to_str().unwrap()])
                    .output()?;
                
                if !output.status.success() {
                    // Refresh the agents list
                    self.refresh_agents()?;
                }
            }
            DetailAction::Enable => {
                let output = std::process::Command::new("launchctl")
                    .args(["enable", &format!("gui/{}/{}",  unsafe { libc::getuid() }, &agent.label)])
                    .output()?;
                
                if !output.status.success() {
                    // Refresh the agents list
                    self.refresh_agents()?;
                }
            }
            DetailAction::Disable => {
                let output = std::process::Command::new("launchctl")
                    .args(["disable", &format!("gui/{}/{}",  unsafe { libc::getuid() }, &agent.label)])
                    .output()?;
                
                if !output.status.success() {
                    // Refresh the agents list
                    self.refresh_agents()?;
                }
            }
            DetailAction::Unload => {
                let output = std::process::Command::new("launchctl")
                    .args(["bootout", &format!("gui/{}/{}", unsafe { libc::getuid() }, &agent.label)])
                    .output()?;
                
                if !output.status.success() {
                    // Refresh the agents list
                    self.refresh_agents()?;
                }
            }
            DetailAction::Back => {
                self.current_screen = Screen::Main;
            }
        }

        Ok(())
    }

    pub fn on_tick(&mut self) -> Result<()> {
        // Refresh the status of the agents periodically
        self.refresh_agents()?;
        Ok(())
    }

    pub fn new_agent(&mut self) {
        self.current_screen = Screen::NewAgent;
        self.new_agent_form = NewAgentForm::default();
        self.new_agent_form_field = NewAgentFormField::Domain;
        self.new_agent_form.input_mode = true;
    }

    pub fn next_form_field(&mut self) {
        self.new_agent_form.input_mode = true;
        self.new_agent_form.show_suggestions = false;
        self.new_agent_form.path_suggestions.clear();
        
        self.new_agent_form_field = match self.new_agent_form_field {
            NewAgentFormField::Domain => NewAgentFormField::Program,
            NewAgentFormField::Program => NewAgentFormField::Arguments,
            NewAgentFormField::Arguments => NewAgentFormField::KeepAlive,
            NewAgentFormField::KeepAlive => NewAgentFormField::StdoutPath,
            NewAgentFormField::StdoutPath => NewAgentFormField::StderrPath,
            NewAgentFormField::StderrPath => NewAgentFormField::UseInterval,
            NewAgentFormField::UseInterval => {
                if self.new_agent_form.use_interval {
                    NewAgentFormField::Interval
                } else {
                    NewAgentFormField::CreateButton
                }
            }
            NewAgentFormField::Interval => NewAgentFormField::CreateButton,
            NewAgentFormField::CreateButton => NewAgentFormField::CancelButton,
            NewAgentFormField::CancelButton => NewAgentFormField::Domain,
        };

        // When moving to a path field, update path suggestions
        match self.new_agent_form_field {
            NewAgentFormField::Program | NewAgentFormField::StdoutPath | NewAgentFormField::StderrPath => {
                self.update_path_suggestions();
            }
            _ => {}
        }
        
        // When leaving the arguments field, exit edit mode
        if self.new_agent_form_field != NewAgentFormField::Arguments {
            if !self.new_agent_form.argument_input.value().is_empty() {
                self.new_agent_form.arguments.push(self.new_agent_form.argument_input.value().to_string());
                self.new_agent_form.argument_input = tui_textarea::Input::default();
            }
            self.edit_mode = false;
        }
        
        // When entering a button, disable input mode
        if self.new_agent_form_field == NewAgentFormField::CreateButton || 
           self.new_agent_form_field == NewAgentFormField::CancelButton {
            self.new_agent_form.input_mode = false;
        }
    }

    pub fn previous_form_field(&mut self) {
        self.new_agent_form.input_mode = true;
        self.new_agent_form.show_suggestions = false;
        self.new_agent_form.path_suggestions.clear();
        
        self.new_agent_form_field = match self.new_agent_form_field {
            NewAgentFormField::Domain => NewAgentFormField::CancelButton,
            NewAgentFormField::Program => NewAgentFormField::Domain,
            NewAgentFormField::Arguments => NewAgentFormField::Program,
            NewAgentFormField::KeepAlive => NewAgentFormField::Arguments,
            NewAgentFormField::StdoutPath => NewAgentFormField::KeepAlive,
            NewAgentFormField::StderrPath => NewAgentFormField::StdoutPath,
            NewAgentFormField::UseInterval => NewAgentFormField::StderrPath,
            NewAgentFormField::Interval => NewAgentFormField::UseInterval,
            NewAgentFormField::CreateButton => {
                if self.new_agent_form.use_interval {
                    NewAgentFormField::Interval
                } else {
                    NewAgentFormField::UseInterval
                }
            }
            NewAgentFormField::CancelButton => NewAgentFormField::CreateButton,
        };
        
        // When moving to a path field, update path suggestions
        match self.new_agent_form_field {
            NewAgentFormField::Program | NewAgentFormField::StdoutPath | NewAgentFormField::StderrPath => {
                self.update_path_suggestions();
            }
            _ => {}
        }

        // When leaving the arguments field, exit edit mode
        if self.new_agent_form_field != NewAgentFormField::Arguments {
            self.edit_mode = false;
        }
        
        // When entering a button, disable input mode
        if self.new_agent_form_field == NewAgentFormField::CreateButton || 
           self.new_agent_form_field == NewAgentFormField::CancelButton {
            self.new_agent_form.input_mode = false;
        }
    }
    
    // Handle keyboard input for the current field
    pub fn handle_input_event(&mut self, event: Event) -> bool {
        if self.new_agent_form.input_mode {
            match self.new_agent_form_field {
                NewAgentFormField::Domain => {
                    if self.new_agent_form.domain_input.handle_event(&event).is_some() {
                        return true;
                    }
                },
                NewAgentFormField::Program => {
                    if self.new_agent_form.show_suggestions {
                        // Handle suggestion selection
                        if let Event::Key(key) = event {
                            match key.code {
                                KeyCode::Up => {
                                    if self.new_agent_form.selected_suggestion > 0 {
                                        self.new_agent_form.selected_suggestion -= 1;
                                    } else if !self.new_agent_form.path_suggestions.is_empty() {
                                        self.new_agent_form.selected_suggestion = self.new_agent_form.path_suggestions.len() - 1;
                                    }
                                    return true;
                                },
                                KeyCode::Down => {
                                    if !self.new_agent_form.path_suggestions.is_empty() {
                                        self.new_agent_form.selected_suggestion = 
                                            (self.new_agent_form.selected_suggestion + 1) % self.new_agent_form.path_suggestions.len();
                                    }
                                    return true;
                                },
                                KeyCode::Enter | KeyCode::Tab => {
                                    if !self.new_agent_form.path_suggestions.is_empty() {
                                        let selected = self.new_agent_form.path_suggestions[self.new_agent_form.selected_suggestion].clone();
                                        self.new_agent_form.program_input = tui_textarea::Input::from(selected);
                                        self.new_agent_form.show_suggestions = false;
                                        return true;
                                    }
                                },
                                KeyCode::Esc => {
                                    self.new_agent_form.show_suggestions = false;
                                    return true;
                                },
                                _ => {}
                            }
                        }
                    }
                    
                    if let Event::Key(KeyEvent { code: KeyCode::Tab, .. }) = event {
                        // Show path suggestions
                        self.update_path_suggestions();
                        self.new_agent_form.show_suggestions = true;
                        self.new_agent_form.selected_suggestion = 0;
                        return true;
                    }
                    
                    if self.new_agent_form.program_input.handle_event(&event).is_some() {
                        return true;
                    }
                },
                NewAgentFormField::Arguments => {
                    if self.edit_mode {
                        if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event {
                            self.edit_mode = false;
                            return true;
                        }
                        
                        if let Event::Key(KeyEvent { code: KeyCode::Char(' '), .. }) = event {
                            if !self.new_agent_form.argument_input.value().is_empty() {
                                self.new_agent_form.arguments.push(self.new_agent_form.argument_input.value().to_string());
                                self.new_agent_form.argument_input = tui_textarea::Input::default();
                            }
                            return true;
                        }
                        
                        if self.new_agent_form.argument_input.handle_event(&event).is_some() {
                            return true;
                        }
                    } else if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event {
                        self.edit_mode = true;
                        return true;
                    }
                },
                NewAgentFormField::StdoutPath | NewAgentFormField::StderrPath => {
                    let input = if self.new_agent_form_field == NewAgentFormField::StdoutPath {
                        &mut self.new_agent_form.stdout_path_input
                    } else {
                        &mut self.new_agent_form.stderr_path_input
                    };
                    
                    if self.new_agent_form.show_suggestions {
                        if let Event::Key(key) = event {
                            match key.code {
                                KeyCode::Up => {
                                    if self.new_agent_form.selected_suggestion > 0 {
                                        self.new_agent_form.selected_suggestion -= 1;
                                    } else if !self.new_agent_form.path_suggestions.is_empty() {
                                        self.new_agent_form.selected_suggestion = self.new_agent_form.path_suggestions.len() - 1;
                                    }
                                    return true;
                                },
                                KeyCode::Down => {
                                    if !self.new_agent_form.path_suggestions.is_empty() {
                                        self.new_agent_form.selected_suggestion = 
                                            (self.new_agent_form.selected_suggestion + 1) % self.new_agent_form.path_suggestions.len();
                                    }
                                    return true;
                                },
                                KeyCode::Enter | KeyCode::Tab => {
                                    if !self.new_agent_form.path_suggestions.is_empty() {
                                        let selected = self.new_agent_form.path_suggestions[self.new_agent_form.selected_suggestion].clone();
                                        *input = tui_textarea::Input::from(selected);
                                        self.new_agent_form.show_suggestions = false;
                                        return true;
                                    }
                                },
                                KeyCode::Esc => {
                                    self.new_agent_form.show_suggestions = false;
                                    return true;
                                },
                                _ => {}
                            }
                        }
                    }
                    
                    if let Event::Key(KeyEvent { code: KeyCode::Tab, .. }) = event {
                        self.update_path_suggestions();
                        self.new_agent_form.show_suggestions = true;
                        self.new_agent_form.selected_suggestion = 0;
                        return true;
                    }
                    
                    if input.handle_event(&event).is_some() {
                        return true;
                    }
                },
                NewAgentFormField::Interval => {
                    match event {
                        Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) if c.is_ascii_digit() => {
                            self.new_agent_form.interval_input.handle_event(&event);
                            if let Ok(val) = self.new_agent_form.interval_input.value().parse::<u32>() {
                                self.new_agent_form.interval = val;
                            }
                            return true;
                        },
                        Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                            self.new_agent_form.interval_input.handle_event(&event);
                            if self.new_agent_form.interval_input.value().is_empty() {
                                self.new_agent_form.interval = 0;
                            } else if let Ok(val) = self.new_agent_form.interval_input.value().parse::<u32>() {
                                self.new_agent_form.interval = val;
                            }
                            return true;
                        },
                        _ => {}
                    }
                },
                NewAgentFormField::KeepAlive => {
                    if let Event::Key(KeyEvent { code: KeyCode::Enter | KeyCode::Char(' '), .. }) = event {
                        self.new_agent_form.keep_alive = !self.new_agent_form.keep_alive;
                        return true;
                    }
                },
                NewAgentFormField::UseInterval => {
                    if let Event::Key(KeyEvent { code: KeyCode::Enter | KeyCode::Char(' '), .. }) = event {
                        self.new_agent_form.use_interval = !self.new_agent_form.use_interval;
                        return true;
                    }
                },
                _ => {}
            }
        } else {
            // Handle non-input mode
            match self.new_agent_form_field {
                NewAgentFormField::CreateButton => {
                    if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event {
                        return true;
                    }
                },
                NewAgentFormField::CancelButton => {
                    if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event {
                        return true;
                    }
                },
                _ => {
                    // Enter input mode for text fields
                    if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event {
                        self.new_agent_form.input_mode = true;
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    pub fn update_path_suggestions(&mut self) {
        let current_value = match self.new_agent_form_field {
            NewAgentFormField::Program => self.new_agent_form.program_input.value(),
            NewAgentFormField::StdoutPath => self.new_agent_form.stdout_path_input.value(),
            NewAgentFormField::StderrPath => self.new_agent_form.stderr_path_input.value(),
            _ => return,
        };
        
        self.new_agent_form.path_suggestions = path_utils::find_path_suggestions(current_value);
        self.new_agent_form.selected_suggestion = 0;
    }

    pub fn create_new_agent(&self) -> Result<bool> {
        // Validation
        let domain_str = self.new_agent_form.domain_input.value();
        let program_str = self.new_agent_form.program_input.value();
        
        if domain_str.is_empty() {
            return Ok(false);
        }
        if program_str.is_empty() {
            return Ok(false);
        }

        let domain = if !domain_str.contains('.') {
            format!("com.user.{}", domain_str)
        } else {
            domain_str.to_string()
        };

        // Create the plist file
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let launch_agents_dir = home_dir.join("Library/LaunchAgents");

        // Ensure the directory exists
        std::fs::create_dir_all(&launch_agents_dir)?;

        let plist_path = launch_agents_dir.join(format!("{}.plist", domain));
        
        create_launch_agent(
            &plist_path,
            &domain,
            &program_str,
            &self.new_agent_form.arguments,
            self.new_agent_form.keep_alive,
            &self.new_agent_form.stdout_path_input.value(),
            &self.new_agent_form.stderr_path_input.value(),
            self.new_agent_form.use_interval,
            self.new_agent_form.interval,
        )?;

        // Load the agent
        let output = std::process::Command::new("launchctl")
            .args(["bootstrap", &format!("gui/{}", unsafe { libc::getuid() }), plist_path.to_str().unwrap()])
            .output()?;

        Ok(output.status.success())
    }
}
