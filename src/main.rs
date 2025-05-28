mod map;
mod robot;
mod ui;
mod station; // Add station module

use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};

use map::Map;
use robot::Robot;
use ui::UI;
use crate::station::Station; // Add import for Station

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize user interface
    let mut ui = UI::new()?;

    // Generate random seed for the map
    let seed = rand::thread_rng().gen();

    // Get terminal size from UI
    let terminal_size = ui.get_terminal_size()?;
    let map_width = terminal_size.width as usize;
    // Adjust map_height to accommodate the new layout in ui.rs (map + 3 lines for stats/controls)
    let map_height = terminal_size.height.saturating_sub(10) as usize; // Adjusted for 9 lines panel + 1 map border
    let mut map = Map::new(map_width, map_height, seed);

    // --- Station Placement Logic ---
    let mut station_x = map_width / 2;
    let mut station_y = map_height / 2;
    let mut station_placement_attempts = 0;
    loop {
        if let Some(cell) = map.get_cell(station_x, station_y) {
            if cell.cell_type != map::CellType::Obstacle {
                break; // Found a non-obstacle spot for the station
            }
        }
        // Try a new spot if current is an obstacle or out of bounds (though get_cell handles out of bounds)
        station_x = (map_width / 2 + station_placement_attempts) % map_width;
        station_y = (map_height / 2 + station_placement_attempts / map_width) % map_height;
        station_placement_attempts += 1;
        if station_placement_attempts > map_width * map_height { // Safety break
            eprintln!("Could not find a non-obstacle position for the station. Placing at (0,0) as fallback.");
            station_x = 0;
            station_y = 0;
            // Ensure (0,0) is not an obstacle, or have a more robust fallback.
            // For simplicity, we might just overwrite it or require map generation to leave (0,0) clear.
            if let Some(cell_mut) = map.get_cell_mut(0,0) { // Example: Force (0,0) to be Empty
                cell_mut.cell_type = map::CellType::Empty;
            }
            break;
        }
    }
    let mut station = Station::new(station_x, station_y); // Create station at the validated non-obstacle position

    // --- Robot Placement Logic (using final station coordinates) ---
    let mut robot_x = station.x; // Use final station.x
    let mut robot_y = station.y.saturating_sub(1); // Try to place above station initially
    if robot_y == station.y { // If above is same as station (e.g. station_y is 0)
        robot_y = station.y.saturating_add(1);
        if robot_y >= map_height { // If below is out of bounds, try left/right or more complex logic
            robot_x = station.x.saturating_sub(1);
            robot_y = station.y;
            if robot_x == station.x { robot_x = station.x.saturating_add(1);}
        }
    }
    // Ensure robot_x and robot_y are within map bounds if changed
    if robot_x >= map_width { robot_x = map_width.saturating_sub(1);}
    if robot_y >= map_height { robot_y = map_height.saturating_sub(1);}

    let mut robot_placement_attempts = 0;
    loop {
        let is_on_station = robot_x == station.x && robot_y == station.y;
        let mut is_on_obstacle = false;
        if let Some(cell) = map.get_cell(robot_x, robot_y) {
            if cell.cell_type == map::CellType::Obstacle {
                is_on_obstacle = true;
            }
        } else { // Spot is out of bounds, definitely need to try a new one
            is_on_obstacle = true; // Treat as an invalid spot
        }

        if !is_on_station && !is_on_obstacle {
            break; // Found a valid spot for the robot
        }

        // Try a new spot for the robot
        // Simple linear probing for robot placement, can be improved
        robot_x = (station.x + robot_placement_attempts) % map_width;
        robot_y = (station.y + robot_placement_attempts / map_width) % map_height;
        robot_placement_attempts += 1;

        if robot_placement_attempts > map_width * map_height { // Safety break
            let (fallback_x, fallback_y) = find_clear_spot_for_robot(&map, station.x, station.y);
            robot_x = fallback_x;
            robot_y = fallback_y;
            eprintln!("Could not find ideal spot for robot, using fallback: ({}, {}).", robot_x, robot_y);
            break;
        }
    }
    let mut robot = Robot::new(robot_x, robot_y);

    // Main loop
    let mut running = true;
    let frame_time = Duration::from_millis(100);

    while running && robot.is_active() {
        let frame_start = Instant::now();

        // Handle user input
        running = ui.handle_input(&mut robot, &mut map)?;

        // --- Robot and Station Interaction Logic ---
        if robot.x == station.x && robot.y == station.y {
            // 1. Unload resources
            let (energy_payload, minerals_payload, science_payload) = robot.unload_payload();
            if energy_payload > 0 || minerals_payload > 0 || science_payload > 0 {
                station.collect_resources(energy_payload, minerals_payload, science_payload);
            }

            // 2. Share map data
            let updates = robot.get_exploration_updates();
            if !updates.is_empty() {
                station.share_data(&updates);
            }
        }

        // --- Station decides to create robots ---
        if station.should_create_robot() {
            let new_robot_start_x = station.x;
            let new_robot_start_y = station.y.saturating_sub(1);

            if let Some(cell) = map.get_cell(new_robot_start_x, new_robot_start_y) {
                 if cell.cell_type != map::CellType::Obstacle && !(new_robot_start_x == station.x && new_robot_start_y == station.y) {
                    if station.create_robot(new_robot_start_x, new_robot_start_y) {
                    }
                 } else {
                 }
            } else {
            }
        }

        // Display the map, robot, and station
        ui.render(&map, &robot, &station)?;

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

// Helper function to find a clear spot for the robot
// This is a simple implementation and might need to be more robust
fn find_clear_spot_for_robot(map: &Map, station_x: usize, station_y: usize) -> (usize, usize) {
    for r_y in 0..map.height {
        for r_x in 0..map.width {
            if let Some(cell) = map.get_cell(r_x, r_y) {
                if cell.cell_type != map::CellType::Obstacle && (r_x != station_x || r_y != station_y) {
                    return (r_x, r_y);
                }
            }
        }
    }
    (0, 0) // Default fallback if no clear spot is found (should ideally not happen)
}
