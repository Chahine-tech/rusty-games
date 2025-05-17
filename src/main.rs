mod map;
mod robot;
mod ui;

use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};

use map::Map;
use robot::Robot;
use ui::UI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize user interface
    let mut ui = UI::new()?;

    // Generate random seed for the map
    let seed = rand::thread_rng().gen();

    // Create a map that fills the terminal
    let map_width = ui.width;
    let map_height = ui.height - 2; // Reserve space for bottom information
    let mut map = Map::new(map_width, map_height, seed);

    // Find a valid position for the robot (not on an obstacle)
    let mut robot_x = map_width / 2;
    let mut robot_y = map_height / 2;

    // S'assurer que le robot ne commence pas sur un obstacle
    while let Some(cell) = map.get_cell(robot_x, robot_y) {
        if let map::CellType::Obstacle = cell.cell_type {
            robot_x = (robot_x + 1) % map_width;
            robot_y = (robot_y + 1) % map_height;
        } else {
            break;
        }
    }

    // Create the robot
    let mut robot = Robot::new(robot_x, robot_y);

    // Main loop
    let mut running = true;
    let frame_time = Duration::from_millis(100); // Increase the time between refreshes (was 16ms)

    while running && robot.is_active() {
        let frame_start = Instant::now();

        // Handle user input
        running = ui.handle_input(&mut robot, &mut map)?;

        // Display the map and the robot
        ui.render(&map, &robot)?;

        // Limit the refresh rate
        let elapsed = frame_start.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
    }

    // Clean up and restore the terminal
    ui.cleanup()?;

    if !robot.is_active() {
        println!("The robot has no energy! Exploration finished.");
        println!("Final statistics:");
        println!("Minerals collected: {}", robot.minerals);
        println!("Scientific points: {}", robot.science_points);
    }

    Ok(())
}
