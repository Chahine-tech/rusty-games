mod map;
mod robot;
mod ui;
mod station; // Add station module

use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};

use map::Map;
use robot::{Robot, RobotType}; // Add RobotType import
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

    // Create the first robot - an Explorer
    let mut first_robot_x = station.x;
    let mut first_robot_y = station.y.saturating_sub(1);
    if first_robot_y == station.y {
        first_robot_y = station.y.saturating_add(1);
        if first_robot_y >= map_height {
            first_robot_x = station.x.saturating_sub(1);
            first_robot_y = station.y;
            if first_robot_x == station.x { 
                first_robot_x = station.x.saturating_add(1);
            }
        }
    }
    
    if first_robot_x >= map_width { 
        first_robot_x = map_width.saturating_sub(1);
    }
    if first_robot_y >= map_height { 
        first_robot_y = map_height.saturating_sub(1);
    }

    let mut robot_placement_attempts = 0;
    loop {
        let is_on_station = first_robot_x == station.x && first_robot_y == station.y;
        let mut is_on_obstacle = false;
        if let Some(cell) = map.get_cell(first_robot_x, first_robot_y) {
            if cell.cell_type == map::CellType::Obstacle {
                is_on_obstacle = true;
            }
        } else {
            is_on_obstacle = true;
        }

        if !is_on_station && !is_on_obstacle {
            break;
        }

        first_robot_x = (station.x + robot_placement_attempts) % map_width;
        first_robot_y = (station.y + robot_placement_attempts / map_width) % map_height;
        robot_placement_attempts += 1;

        if robot_placement_attempts > map_width * map_height {
            let (fallback_x, fallback_y) = find_clear_spot_for_robot(&map, station.x, station.y);
            first_robot_x = fallback_x;
            first_robot_y = fallback_y;
            eprintln!("Could not find ideal spot for first robot, using fallback: ({}, {}).", first_robot_x, first_robot_y);
            break;
        }
    }
    
    // Create initial robots - prioritize explorers for better coverage
    let robot_types = [
        RobotType::Explorer,
        RobotType::Explorer,     
        RobotType::Explorer,     
        RobotType::Explorer,     // Additional explorer
        RobotType::Explorer,     // Additional explorer
        RobotType::Explorer,     // Additional explorer
        RobotType::EnergyCollector, 
        RobotType::MineralCollector,
        RobotType::Scientist,
    ];

    // Define starting directions to spread robots out - more directions for more robots
    let start_directions = [
        (0, -8),    // North (further)
        (8, 0),     // East (further)
        (0, 8),     // South (further)
        (-8, 0),    // West (further)
        (6, -6),    // Northeast (further)
        (-6, 6),    // Southwest (further)
        (6, 6),     // Southeast (further)
        (-6, -6),   // Northwest (further)
        (0, -12),   // Far North
    ];

    for (i, robot_type) in robot_types.iter().enumerate() {
        let (robot_x, robot_y) = if i == 0 {
            // Use the calculated position for the first robot
            (first_robot_x, first_robot_y)
        } else {
            // Try to place robots in different directions from station
            let (dx, dy) = start_directions[i % start_directions.len()];
            let target_x = (station.x as i32 + dx).max(0).min(map_width as i32 - 1) as usize;
            let target_y = (station.y as i32 + dy).max(0).min(map_height as i32 - 1) as usize;
            
            // Find nearest clear spot to the target direction
            find_clear_spot_near_target(&map, target_x, target_y, &station.robots)
        };
        
        // Create robot directly and add to station (bypass resource cost for initial robots)
        let robot = Robot::new_with_type(robot_x, robot_y, *robot_type);
        station.robots.push(robot);
    }

    // Main loop
    let mut running = true;
    let frame_time = Duration::from_millis(100); // Even faster updates for more aggressive exploration

    while running {
        let frame_start = Instant::now();

        // Handle user input (only quit in autonomous mode)
        running = ui.handle_input()?;

        // Update all robots autonomously
        for i in 0..station.robots.len() {
            // Create a slice of other robots (excluding the current one)
            let (left, right) = station.robots.split_at_mut(i);
            let (current, right) = right.split_first_mut().unwrap();
            let other_robots: Vec<_> = left.iter().chain(right.iter()).cloned().collect();
            
            current.autonomous_update(&mut map, station.x, station.y, &other_robots);
        }

        // Handle robot-station interactions
        let mut robots_to_update = Vec::new();
        for i in 0..station.robots.len() {
            let robot = &station.robots[i];
            if robot.x == station.x && robot.y == station.y {
                robots_to_update.push(i);
            }
        }

        // Process interactions for robots at station
        for &robot_index in &robots_to_update {
            // 1. Unload resources
            let (energy_payload, minerals_payload, science_payload) = station.robots[robot_index].unload_payload();
            if energy_payload > 0 || minerals_payload > 0 || science_payload > 0 {
                station.collect_resources(energy_payload, minerals_payload, science_payload);
            }

            // 2. Share map data
            let updates = station.robots[robot_index].get_exploration_updates();
            if !updates.is_empty() {
                station.share_data(&updates);
            }

            // 3. Refuel robot at station (consume station energy)
            let refuel_cost = robot::INITIAL_ROBOT_ENERGY.saturating_sub(station.robots[robot_index].energy);
            if refuel_cost > 0 && station.energy >= refuel_cost {
                station.energy -= refuel_cost;
                station.robots[robot_index].energy = robot::INITIAL_ROBOT_ENERGY;
            }

            // 4. Update robot state to continue exploring
            station.robots[robot_index].state = robot::RobotState::Exploring;
        }

        // Handle dead robots - respawn them at the station (if station has energy)
        for robot in &mut station.robots {
            if robot.energy == 0 {
                robot.x = station.x;
                robot.y = station.y;
                robot.state = robot::RobotState::AtStation;
                robot.steps_since_last_find = 0;
                
                // Respawn robot only if station has enough energy
                if station.energy >= robot::INITIAL_ROBOT_ENERGY {
                    station.energy -= robot::INITIAL_ROBOT_ENERGY;
                    robot.energy = robot::INITIAL_ROBOT_ENERGY;
                }
            }
        }

        // Station decides to create new robots
        if station.should_create_robot() {
            let (new_robot_x, new_robot_y) = find_clear_spot_for_robot(&map, station.x, station.y);
            
            if let Some(cell) = map.get_cell(new_robot_x, new_robot_y) {
                if cell.cell_type != map::CellType::Obstacle && !(new_robot_x == station.x && new_robot_y == station.y) {
                    station.create_robot(new_robot_x, new_robot_y);
                }
            }
        }

        // Display the map and station
        ui.render(&map, &station)?;

        // Limit the refresh rate
        let elapsed = frame_start.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
    }

    // Clean up and restore the terminal
    ui.cleanup()?;

    println!("Autonomous exploration simulation ended.");
    println!("Final station statistics:");
    println!("Station Energy: {}", station.energy);
    println!("Station Minerals: {}", station.minerals);
    println!("Station Science Points: {}", station.science_points);
    println!("Total Robots Created: {}", station.robots.len());

    Ok(())
}

// Helper function to find a clear spot for the robot
// Tries to find spots in expanding circles around the station
fn find_clear_spot_for_robot(map: &Map, station_x: usize, station_y: usize) -> (usize, usize) {
    find_clear_spot_for_robot_avoiding_others(map, station_x, station_y, &[])
}

// Helper function to find a clear spot near a target position
fn find_clear_spot_near_target(map: &Map, target_x: usize, target_y: usize, existing_robots: &[Robot]) -> (usize, usize) {
    // First try the exact target position
    if let Some(cell) = map.get_cell(target_x, target_y) {
        let position_occupied = existing_robots.iter().any(|r| r.x == target_x && r.y == target_y);
        if cell.cell_type != map::CellType::Obstacle && !position_occupied {
            return (target_x, target_y);
        }
    }
    
    // Try positions in expanding circles around target
    for radius in 1..=5 {
        for dx in -(radius as i32)..=(radius as i32) {
            for dy in -(radius as i32)..=(radius as i32) {
                if dx.abs() != radius && dy.abs() != radius {
                    continue; // Only check the perimeter
                }
                
                let new_x = (target_x as i32 + dx) as usize;
                let new_y = (target_y as i32 + dy) as usize;
                
                if new_x < map.width && new_y < map.height {
                    if let Some(cell) = map.get_cell(new_x, new_y) {
                        let position_occupied = existing_robots.iter().any(|robot| robot.x == new_x && robot.y == new_y);
                        if cell.cell_type != map::CellType::Obstacle && !position_occupied {
                            return (new_x, new_y);
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to general search
    find_clear_spot_for_robot_avoiding_others(map, target_x, target_y, existing_robots)
}

// Helper function to find a clear spot for a robot, avoiding other robots
fn find_clear_spot_for_robot_avoiding_others(map: &Map, station_x: usize, station_y: usize, existing_robots: &[Robot]) -> (usize, usize) {
    // First try positions around the station in a spiral pattern
    for radius in 1..=5 {
        for dx in -(radius as i32)..=(radius as i32) {
            for dy in -(radius as i32)..=(radius as i32) {
                if dx.abs() != radius && dy.abs() != radius {
                    continue; // Only check the perimeter of each radius
                }
                
                let new_x = (station_x as i32 + dx) as usize;
                let new_y = (station_y as i32 + dy) as usize;
                
                if new_x < map.width && new_y < map.height {
                    // Check if position is occupied by station
                    if new_x == station_x && new_y == station_y {
                        continue;
                    }
                    
                    // Check if position is occupied by existing robots
                    let position_occupied = existing_robots.iter().any(|r| r.x == new_x && r.y == new_y);
                    if position_occupied {
                        continue;
                    }
                    
                    // Check if position is obstacle
                    if let Some(cell) = map.get_cell(new_x, new_y) {
                        if cell.cell_type != map::CellType::Obstacle {
                            return (new_x, new_y);
                        }
                    }
                }
            }
        }
    }
    
    // Fallback: scan the entire map
    for r_y in 0..map.height {
        for r_x in 0..map.width {
            if let Some(cell) = map.get_cell(r_x, r_y) {
                let position_occupied = (r_x == station_x && r_y == station_y) ||
                    existing_robots.iter().any(|robot| robot.x == r_x && robot.y == r_y);
                
                if cell.cell_type != map::CellType::Obstacle && !position_occupied {
                    return (r_x, r_y);
                }
            }
        }
    }
    (0, 0) // Default fallback if no clear spot is found (should ideally not happen)
}
