use std::collections::HashMap;
use crate::map::{CellType, RobotExplorationUpdate}; // Updated import
use crate::robot::Robot; // Import the Robot struct

const ROBOT_ENERGY_COST: u32 = 100;
const ROBOT_MINERAL_COST: u32 = 50;

pub struct Station {
    pub x: usize, // Added x coordinate
    pub y: usize, // Added y coordinate
    pub energy: u32,
    pub minerals: u32,
    pub science_points: u32,
    pub known_map: HashMap<(usize, usize), CellType>, // Station's knowledge of the map
    pub robots: Vec<Robot>, // List of robots managed by the station
}

impl Station {
    pub fn new(x: usize, y: usize) -> Self { // Added x, y parameters
        Self {
            x, // Initialize x
            y, // Initialize y
            energy: 1000, // Initial energy for the station
            minerals: 500, // Initial minerals for the station
            science_points: 0,
            known_map: HashMap::new(), // Initialize with an empty map
            robots: Vec::new(), // Initialize with an empty list of robots
        }
    }

    // Method to collect resources from a robot
    pub fn collect_resources(&mut self, energy: u32, minerals: u32, science: u32) {
        self.energy += energy;
        self.minerals += minerals;
        self.science_points += science;
        // Potentially add logging or events here
    }

    // Method to consume resources (e.g., for creating robots)
    pub fn consume_resources(&mut self, energy_cost: u32, mineral_cost: u32) -> bool {
        if self.energy >= energy_cost && self.minerals >= mineral_cost {
            self.energy -= energy_cost;
            self.minerals -= mineral_cost;
            true
        } else {
            false
        }
    }

    // Updated robot creation logic
    pub fn should_create_robot(&self) -> bool {
        // Constants for robot creation strategy
        const ROBOT_CREATION_MINERAL_BUFFER: u32 = 150;
        const ROBOT_CREATION_ENERGY_BUFFER: u32 = 400;
        const MAX_ROBOT_COUNT: usize = 5;
        // Minimum number of known valuable resource locations to justify building a new robot
        const MIN_KNOWN_UNTAPPED_VALUABLE_CELLS_FOR_NEW_ROBOT: usize = 3;

        // 1. Check if maximum robot capacity has been reached
        if self.robots.len() >= MAX_ROBOT_COUNT {
            return false;
        }

        // 2. Check if the station has enough resources (including a buffer)
        if self.minerals < ROBOT_MINERAL_COST + ROBOT_CREATION_MINERAL_BUFFER ||
           self.energy < ROBOT_ENERGY_COST + ROBOT_CREATION_ENERGY_BUFFER {
            return false;
        }

        // 3. Analyze the known map for untapped resources
        let mut known_untapped_valuable_cells = 0;
        for cell_type in self.known_map.values() {
            match cell_type {
                CellType::Energy(amount) if *amount > 0 => known_untapped_valuable_cells += 1,
                CellType::Mineral(amount) if *amount > 0 => known_untapped_valuable_cells += 1,
                // Optionally, consider SciencePoints as valuable targets if strategy dictates
                // CellType::SciencePoint => known_untapped_valuable_cells += 1,
                _ => {}
            }
        }

        // Only build if there are enough known targets to make a new robot worthwhile
        if known_untapped_valuable_cells < MIN_KNOWN_UNTAPPED_VALUABLE_CELLS_FOR_NEW_ROBOT {
            // Alternative dynamic threshold:
            // if known_untapped_valuable_cells < (self.robots.len() + 1) * TARGETS_PER_ROBOT_THRESHOLD {
            return false;
        }

        true // All conditions met, station should create a robot
    }

    // Method to create a new robot
    // Takes starting coordinates for the new robot
    pub fn create_robot(&mut self, start_x: usize, start_y: usize) -> bool {
        if self.consume_resources(ROBOT_ENERGY_COST, ROBOT_MINERAL_COST) {
            let new_robot = Robot::new(start_x, start_y);
            self.robots.push(new_robot);
            // Potentially log robot creation
            true
        } else {
            // Potentially log failure due to insufficient resources
            false
        }
    }

    // Helper method to analyze current map data
    fn analyze_map_data(&self) {
        // Example: Count valuable cells (energy, minerals, science points)
        // This is a placeholder for more sophisticated analysis
        let mut _valuable_cells_count = 0;
        for ((_x, _y), cell_type) in &self.known_map {
            match cell_type {
                CellType::Energy(amount) if *amount > 0 => _valuable_cells_count += 1,
                CellType::Mineral(amount) if *amount > 0 => _valuable_cells_count += 1,
                _ => {}
            }
        }
        // This function is a placeholder for more complex analysis.
        // For now, its execution signifies that analysis is "triggered".
        // In a real game, it might update station goals, log information,
        // or set flags that influence other decisions.
        // Example: if _valuable_cells_count > 10 { /* log high resource density */ } // Also prefixed here if used in example
    }

    // Method to integrate exploration data from a robot
    pub fn share_data(&mut self, data_from_robot: &RobotExplorationUpdate) {
        for ((x, y), cell_type) in data_from_robot {
            // Simple merge: last write wins.
            // Assumes CellType is Clone.
            self.known_map.insert((*x, *y), cell_type.clone());
        }
        self.analyze_map_data(); // Trigger analysis based on the new map data.
                                 // Decisions (like robot creation) will use this updated map.
    }

    pub fn display_stats(&self) -> String {
        format!(
            "Station @ ({}, {}) => Energy: {}, Minerals: {}, Science: {}, Robots: {}",
            self.x, self.y, self.energy, self.minerals, self.science_points, self.robots.len()
        )
    }
}

// The RobotExplorationUpdate type and known_map field handle shared map information
