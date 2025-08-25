use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{AppState, DetailAction, NewAgentFormField, Screen};

pub fn render(f: &mut Frame, state: &mut AppState) {
    match state.current_screen {
        Screen::Main => draw_main_screen(f, state),
        Screen::Detail => draw_detail_screen(f, state),
        Screen::NewAgent => draw_new_agent_screen(f, state),
    }
}

fn draw_main_screen(f: &mut Frame, state: &mut AppState) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Draw title
    let title = Paragraph::new("LaunchIt - Launch Agent Manager")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Draw list of agents
    let mut items = Vec::new();
    for agent in &state.agents {
        let status_style = if agent.last_exit_status.unwrap_or(0) != 0 {
            Style::default().fg(Color::Red)
        } else if agent.pid.is_some() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let pid_text = match agent.pid {
            Some(pid) => format!("[{}]", pid),
            None => "[stopped]".to_string(),
        };

        let status_text = match agent.last_exit_status {
            Some(0) => "".to_string(),
            Some(code) => format!(" (exit code: {})", code),
            None => "".to_string(),
        };

        let line = Line::from(vec![
            Span::raw(format!("{:<20}", agent.label)),
            Span::styled(format!("{:<10}", pid_text), status_style),
            Span::raw(status_text),
        ]);

        items.push(ListItem::new(line));
    }

    // We'll create a custom list with highlight style

    // Render the list with highlighting based on selection
    let mut items_with_highlight: Vec<ListItem> = Vec::new();
    
    for (i, agent) in state.agents.iter().enumerate() {
        let status_style = if agent.last_exit_status.unwrap_or(0) != 0 {
            Style::default().fg(Color::Red)
        } else if agent.pid.is_some() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let pid_text = match agent.pid {
            Some(pid) => format!("[{}]", pid),
            None => "[stopped]".to_string(),
        };

        let status_text = match agent.last_exit_status {
            Some(0) => "".to_string(),
            Some(code) => format!(" (exit code: {})", code),
            None => "".to_string(),
        };
        
        let prefix = if state.selected == Some(i) {
            Span::styled(">> ", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("   ")
        };
        
        let line = Line::from(vec![
            prefix,
            Span::raw(format!("{:<20}", agent.label)),
            Span::styled(format!("{:<10}", pid_text), status_style),
            Span::raw(status_text),
        ]);
        
        items_with_highlight.push(ListItem::new(line));
    }
    
    let list = List::new(items_with_highlight)
        .block(Block::default().borders(Borders::NONE));
        
    f.render_widget(list, chunks[1]);

    // Draw footer
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("Press "),
        Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to create new agent, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to view details, "),
        Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to refresh, "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to quit"),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

fn draw_detail_screen(f: &mut Frame, state: &mut AppState) {
    if state.agent_detail_index >= state.agents.len() {
        state.current_screen = Screen::Main;
        return;
    }

    let agent = &state.agents[state.agent_detail_index];
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(5),     // Agent details
            Constraint::Length(7),  // Actions
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Draw title
    let title = Paragraph::new(format!("Agent: {}", agent.label))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Draw agent details
    let status_style = if agent.last_exit_status.unwrap_or(0) != 0 {
        Style::default().fg(Color::Red)
    } else if agent.pid.is_some() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let pid_text = match agent.pid {
        Some(pid) => format!("PID: {}", pid),
        None => "Status: Not running".to_string(),
    };

    let status_text = match agent.last_exit_status {
        Some(0) => "Exit Status: Success (0)".to_string(),
        Some(code) => format!("Exit Status: Failed ({})", code),
        None => "Exit Status: Unknown".to_string(),
    };

    let details = vec![
        Line::from(format!("Label: {}", agent.label)),
        Line::from(format!("Path: {}", agent.path.display())),
        Line::from(vec![Span::styled(pid_text, status_style)]),
        Line::from(vec![Span::styled(status_text, status_style)]),
        Line::from(format!("Program: {}", agent.program)),
        Line::from(format!("Arguments: {}", agent.arguments.join(" "))),
    ];

    let details_widget = Paragraph::new(details).block(Block::default().borders(Borders::NONE));
    f.render_widget(details_widget, chunks[1]);

    // Draw actions
    let mut actions = Vec::new();
    for (i, action) in state.detail_actions.iter().enumerate() {
        let text = match action {
            DetailAction::Load => "Load",
            DetailAction::Enable => "Enable",
            DetailAction::Disable => "Disable",
            DetailAction::Unload => "Unload",
            DetailAction::Back => "Back to list",
        };

        let style = if i == state.selected_action {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        actions.push(ListItem::new(Line::from(Span::styled(text, style))));
    }

    let actions_list = List::new(actions).block(Block::default().title("Actions").borders(Borders::ALL));
    f.render_widget(actions_list, chunks[2]);

    // Draw footer
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("Press "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to select action, "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to go back"),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[3]);
}

fn draw_new_agent_screen(f: &mut Frame, state: &mut AppState) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Form
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Draw title
    let title = Paragraph::new("Create New Launch Agent")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Draw form
    let form_area = chunks[1];
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Domain
            Constraint::Length(3), // Program
            Constraint::Length(if state.edit_mode { 6 } else { 3 }), // Arguments
            Constraint::Length(3), // Keep Alive
            Constraint::Length(3), // Stdout Path
            Constraint::Length(3), // Stderr Path
            Constraint::Length(3), // Use Interval
            Constraint::Length(if state.new_agent_form.use_interval { 3 } else { 0 }), // Interval
            Constraint::Length(3), // Buttons
        ])
        .split(form_area);

    // Domain field
    let domain_style = if state.new_agent_form_field == NewAgentFormField::Domain {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let domain_block = Block::default().title("Domain (e.g. com.example.app)").borders(Borders::ALL).style(domain_style);
    let domain_text = Paragraph::new(state.new_agent_form.domain_input.value()).block(domain_block);
    f.render_widget(domain_text, form_chunks[0]);

    // Program field
    let program_style = if state.new_agent_form_field == NewAgentFormField::Program {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let program_block = Block::default().title("Program (absolute path)").borders(Borders::ALL).style(program_style);
    let program_text = Paragraph::new(state.new_agent_form.program_input.value()).block(program_block);
    
    // Show path suggestions if enabled
    if state.new_agent_form.show_suggestions && state.new_agent_form_field == NewAgentFormField::Program {
        let suggestions_area = Rect::new(
            form_chunks[1].x, 
            form_chunks[1].y + form_chunks[1].height,
            form_chunks[1].width,
            std::cmp::min(state.new_agent_form.path_suggestions.len() as u16, 5)
        );
        
        let suggestions: Vec<ListItem> = state.new_agent_form.path_suggestions
            .iter()
            .enumerate()
            .map(|(i, path)| {
                if i == state.new_agent_form.selected_suggestion {
                    ListItem::new(path.clone()).style(Style::default().bg(Color::Gray).fg(Color::Black))
                } else {
                    ListItem::new(path.clone())
                }
            })
            .collect();
            
        let suggestions_list = List::new(suggestions)
            .block(Block::default().borders(Borders::ALL).title("Suggestions"))
            .highlight_style(Style::default().bg(Color::Gray).fg(Color::Black));
            
        if !suggestions_area.height == 0 {
            f.render_widget(suggestions_list, suggestions_area);
        }
    }
    f.render_widget(program_text, form_chunks[1]);

    // Arguments field
    let args_style = if state.new_agent_form_field == NewAgentFormField::Arguments {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let args_block = Block::default()
        .title(if state.edit_mode {
            "Arguments (Space to add, Enter to finish)"
        } else {
            "Arguments (Enter to edit)"
        })
        .borders(Borders::ALL)
        .style(args_style);
    
    let args_content = if state.edit_mode {
        format!(
            "Current arguments: [{}]\nNew argument: {}",
            state.new_agent_form.arguments.join(", "),
            state.new_argument
        )
    } else {
        format!(
            "[{}]",
            state.new_agent_form.arguments.join(", ")
        )
    };
    
    let args_text = Paragraph::new(args_content).block(args_block);
    f.render_widget(args_text, form_chunks[2]);

    // Keep Alive field
    let keep_alive_style = if state.new_agent_form_field == NewAgentFormField::KeepAlive {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let keep_alive_block = Block::default()
        .title("Keep Alive")
        .borders(Borders::ALL)
        .style(keep_alive_style);
    
    let keep_alive_text = Paragraph::new(format!("[{}]", if state.new_agent_form.keep_alive { "X" } else { " " }))
        .block(keep_alive_block);
    f.render_widget(keep_alive_text, form_chunks[3]);

    // Stdout Path field
    let stdout_style = if state.new_agent_form_field == NewAgentFormField::StdoutPath {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let stdout_block = Block::default()
        .title("Standard Output Path")
        .borders(Borders::ALL)
        .style(stdout_style);
    let stdout_text = Paragraph::new(state.new_agent_form.stdout_path_input.value()).block(stdout_block);
    f.render_widget(stdout_text, form_chunks[4]);

    // Stderr Path field
    let stderr_style = if state.new_agent_form_field == NewAgentFormField::StderrPath {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let stderr_block = Block::default()
        .title("Standard Error Path")
        .borders(Borders::ALL)
        .style(stderr_style);
    let stderr_text = Paragraph::new(state.new_agent_form.stderr_path_input.value()).block(stderr_block);
    f.render_widget(stderr_text, form_chunks[5]);

    // Use Interval field
    let use_interval_style = if state.new_agent_form_field == NewAgentFormField::UseInterval {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let use_interval_block = Block::default()
        .title("Use Start Interval")
        .borders(Borders::ALL)
        .style(use_interval_style);
    
    let use_interval_text = Paragraph::new(format!("[{}]", if state.new_agent_form.use_interval { "X" } else { " " }))
        .block(use_interval_block);
    f.render_widget(use_interval_text, form_chunks[6]);

    // Interval field (only shown if use_interval is true)
    if state.new_agent_form.use_interval {
        let interval_style = if state.new_agent_form_field == NewAgentFormField::Interval {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let interval_block = Block::default()
            .title("Interval (seconds)")
            .borders(Borders::ALL)
            .style(interval_style);
        
        let interval_text = Paragraph::new(state.new_agent_form.interval_input.value())
            .block(interval_block);
        f.render_widget(interval_text, form_chunks[7]);
    }

    // Buttons
    let button_area = form_chunks[if state.new_agent_form.use_interval { 8 } else { 7 }];
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(button_area);

    // Create button
    let create_style = if state.new_agent_form_field == NewAgentFormField::CreateButton {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let create_button = Paragraph::new("[ Create ]")
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(create_style));
    f.render_widget(create_button, button_chunks[0]);

    // Cancel button
    let cancel_style = if state.new_agent_form_field == NewAgentFormField::CancelButton {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let cancel_button = Paragraph::new("[ Cancel ]")
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(cancel_style));
    f.render_widget(cancel_button, button_chunks[1]);

    // Draw footer
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("Press "),
        Span::styled("Tab/Up/Down", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to navigate, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to select/edit, "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to cancel"),
    ]))
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
