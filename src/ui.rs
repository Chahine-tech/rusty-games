use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::{stdout, Result};
use std::time::Duration;

use crate::map::Map;
use crate::station::Station; // Add import for Station

// Structure to manage the user interface
pub struct UI {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl UI {
    // Create a new user interface
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        terminal.hide_cursor()?;
        terminal.clear()?;
        Ok(Self { terminal })
    }

    pub fn get_terminal_size(&self) -> Result<Rect> {
        self.terminal.size()
    }

    // Clean up and restore the terminal
    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    // Display the map and the station's information (autonomous mode)
    pub fn render(&mut self, map: &Map, station: &Station) -> Result<()> { // Remove robot parameter
        self.terminal.draw(|frame| {
            let main_layout = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // Map area
                    Constraint::Length(9), // Bottom panel: 3 sections * 3 lines/section = 9 lines
                ])
                .split(frame.size());

            // Render map with all robots
            let mut map_text_lines = Vec::new();
            for y in 0..map.height {
                let mut line = String::new();
                for x in 0..map.width {
                    // Check if any robot is at this position
                    let robot_at_position = station.robots.iter().find(|robot| robot.x == x && robot.y == y);
                    
                    if let Some(robot) = robot_at_position {
                        // Display robot with type-specific symbol
                        let symbol = match robot.robot_type {
                            crate::robot::RobotType::Explorer => 'E',
                            crate::robot::RobotType::EnergyCollector => 'G', // G for enerGy
                            crate::robot::RobotType::MineralCollector => 'M',
                            crate::robot::RobotType::Scientist => 'S',
                        };
                        line.push(symbol);
                    } else if x == station.x && y == station.y { // Check for station position
                        line.push('H'); // 'H' for Home/Station
                    } else if let Some(cell) = map.get_cell(x, y) {
                        let symbol = match cell.cell_type {
                            crate::map::CellType::Empty => " ",
                            crate::map::CellType::Obstacle => "▓",
                            crate::map::CellType::Energy(_) => "e",
                            crate::map::CellType::Mineral(_) => "m",
                            crate::map::CellType::SciencePoint => "s",
                        };
                        line.push_str(symbol);
                    } else {
                        line.push(' '); // Should not happen if map is correctly sized
                    }
                }
                map_text_lines.push(Line::from(line));
            }
            let map_paragraph = Paragraph::new(map_text_lines)
                .block(Block::default().title("Autonomous Robot Swarm").borders(Borders::ALL));
            frame.render_widget(map_paragraph, main_layout[0]);

            // Render stats and info
            let bottom_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Station stats (3 lines: title/border, content, border)
                    Constraint::Length(3), // Swarm stats (3 lines)
                    Constraint::Length(3), // Info (3 lines)
                ])
                .split(main_layout[1]); // Split the 9-line bottom area

            let station_stats_paragraph = Paragraph::new(station.display_stats())
                .block(Block::default().title("Station Stats").borders(Borders::ALL));
            frame.render_widget(station_stats_paragraph, bottom_chunks[0]); // Render in the first 3-line chunk

            let swarm_stats_paragraph = Paragraph::new(station.display_swarm_stats())
                .block(Block::default().title("Swarm Stats").borders(Borders::ALL));
            frame.render_widget(swarm_stats_paragraph, bottom_chunks[1]); // Render in the second 3-line chunk

            let info_paragraph =
                Paragraph::new("Autonomous Mode: Robots explore and collect resources automatically | Q: Quit")
                    .block(Block::default().title("Info").borders(Borders::ALL));
            frame.render_widget(info_paragraph, bottom_chunks[2]); // Render in the third 3-line chunk

        })?;
        Ok(())
    }

    // Wait and process user input (autonomous mode - only quit control)
    pub fn handle_input(&self) -> Result<bool> { // Remove robot and map parameters
        // Increase the waiting time to reduce polling frequency
        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        return Ok(false);
                    }
                    _ => {
                        // Ignore all other inputs in autonomous mode
                    }
                }
            }
        }

        Ok(true)
    }
}
