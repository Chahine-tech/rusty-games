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
use crate::robot::{Direction, Robot};
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

    // Display the map and the robot's information
    pub fn render(&mut self, map: &Map, robot: &Robot, station: &Station) -> Result<()> { // Add station parameter
        self.terminal.draw(|frame| {
            let main_layout = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // Map area
                    Constraint::Length(9), // Bottom panel: 3 sections * 3 lines/section = 9 lines
                ])
                .split(frame.size());

            // Render map
            let mut map_text_lines = Vec::new();
            for y in 0..map.height {
                let mut line = String::new();
                for x in 0..map.width {
                    if x == robot.x && y == robot.y {
                        line.push('R');
                    } else if x == station.x && y == station.y { // Check for station position
                        line.push('H'); // 'H' for Home/Station
                    } else if let Some(cell) = map.get_cell(x, y) {
                        let symbol = match cell.cell_type {
                            crate::map::CellType::Empty => " ",
                            crate::map::CellType::Obstacle => "▓",
                            crate::map::CellType::Energy(_) => "E",
                            crate::map::CellType::Mineral(_) => "M",
                            crate::map::CellType::SciencePoint => "S",
                        };
                        line.push_str(symbol);
                    } else {
                        line.push(' '); // Should not happen if map is correctly sized
                    }
                }
                map_text_lines.push(Line::from(line));
            }
            let map_paragraph = Paragraph::new(map_text_lines)
                .block(Block::default().title("Map").borders(Borders::ALL));
            frame.render_widget(map_paragraph, main_layout[0]);

            // Render stats and controls
            let bottom_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Robot stats (3 lines: title/border, content, border)
                    Constraint::Length(3), // Station stats (3 lines)
                    Constraint::Length(3), // Controls (3 lines)
                ])
                .split(main_layout[1]); // Split the 9-line bottom area

            let robot_stats_paragraph = Paragraph::new(robot.display_stats())
                .block(Block::default().title("Robot Stats").borders(Borders::ALL));
            frame.render_widget(robot_stats_paragraph, bottom_chunks[0]); // Render in the first 3-line chunk

            let station_stats_paragraph = Paragraph::new(station.display_stats())
                .block(Block::default().title("Station Stats").borders(Borders::ALL));
            frame.render_widget(station_stats_paragraph, bottom_chunks[1]); // Render in the second 3-line chunk

            let controls_paragraph =
                Paragraph::new("Controls: Arrows | E: Explore | C: Collect | Q: Quit")
                    .block(Block::default().title("Controls").borders(Borders::ALL));
            frame.render_widget(controls_paragraph, bottom_chunks[2]); // Render in the third 3-line chunk

        })?;
        Ok(())
    }

    // Wait and process a user input
    pub fn handle_input(&self, robot: &mut Robot, map: &mut Map) -> Result<bool> {
        // Increase the waiting time to reduce polling frequency
        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        robot.move_in_direction(Direction::North, map);
                    }
                    KeyCode::Right => {
                        robot.move_in_direction(Direction::East, map);
                    }
                    KeyCode::Down => {
                        robot.move_in_direction(Direction::South, map);
                    }
                    KeyCode::Left => {
                        robot.move_in_direction(Direction::West, map);
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        robot.explore(map);
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        robot.collect_resource(map);
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }

        Ok(true)
    }
}
