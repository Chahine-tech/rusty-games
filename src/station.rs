use std::collections::HashMap;
use crate::map::{CellType, RobotExplorationUpdate}; // Updated import
use crate::robot::{Robot, RobotType}; // Import the Robot struct and RobotType

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
            energy: 2000, // Increased starting energy for better robot support
            minerals: 500, // Also increased starting minerals
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
        const ROBOT_CREATION_MINERAL_BUFFER: u32 = 100; // Reduced buffer to create robots more aggressively
        const ROBOT_CREATION_ENERGY_BUFFER: u32 = 300; // Reduced buffer
        const MAX_ROBOT_COUNT: usize = 12; // Increased from 5 to 12 for better exploration coverage
        // Minimum number of known valuable resource locations to justify building a new robot
        const MIN_KNOWN_UNTAPPED_VALUABLE_CELLS_FOR_NEW_ROBOT: usize = 2; // Reduced threshold

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
                // Also consider SciencePoints as valuable targets
                CellType::SciencePoint => known_untapped_valuable_cells += 1,
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

    // Method to create a new robot with intelligent type selection
    // Takes starting coordinates for the new robot
    pub fn create_robot(&mut self, start_x: usize, start_y: usize) -> bool {
        if self.consume_resources(ROBOT_ENERGY_COST, ROBOT_MINERAL_COST) {
            let robot_type = self.choose_robot_type();
            let new_robot = Robot::new_with_type(start_x, start_y, robot_type);
            self.robots.push(new_robot);
            // Potentially log robot creation
            true
        } else {
            // Potentially log failure due to insufficient resources
            false
        }
    }

    // Intelligent robot type selection based on current needs
    fn choose_robot_type(&self) -> RobotType {
        // Count existing robots by type
        let mut explorer_count = 0;
        let mut energy_collector_count = 0;
        let mut mineral_collector_count = 0;
        let mut scientist_count = 0;

        for robot in &self.robots {
            match robot.robot_type {
                RobotType::Explorer => explorer_count += 1,
                RobotType::EnergyCollector => energy_collector_count += 1,
                RobotType::MineralCollector => mineral_collector_count += 1,
                RobotType::Scientist => scientist_count += 1,
            }
        }

        // Analyze map data to determine priorities
        let mut energy_sources = 0;
        let mut mineral_sources = 0;
        let mut science_sources = 0;
        let mut unexplored_cells = 0;

        for cell_type in self.known_map.values() {
            match cell_type {
                CellType::Energy(amount) if *amount > 0 => energy_sources += 1,
                CellType::Mineral(amount) if *amount > 0 => mineral_sources += 1,
                CellType::SciencePoint => science_sources += 1,
                CellType::Empty => unexplored_cells += 1,
                _ => {}
            }
        }

        // Decision logic based on current situation
        // Always ensure at least one explorer if map is not fully explored
        if explorer_count == 0 || (unexplored_cells > 10 && explorer_count < 2) {
            return RobotType::Explorer;
        }

        // If low on energy and energy sources are available, prioritize energy collectors
        if self.energy < 300 && energy_sources > 0 && energy_collector_count < 2 {
            return RobotType::EnergyCollector;
        }

        // If mineral sources are abundant and we need more minerals
        if mineral_sources > energy_sources && mineral_collector_count < 2 {
            return RobotType::MineralCollector;
        }

        // If science sources are available and we want to maximize science points
        if science_sources > 0 && scientist_count < 1 {
            return RobotType::Scientist;
        }

        // Default to explorer for general exploration
        RobotType::Explorer
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

    // Display swarm statistics
    pub fn display_swarm_stats(&self) -> String {
        if self.robots.is_empty() {
            return "No robots in swarm".to_string();
        }

        // Count robots by type and state
        let mut explorer_count = 0;
        let mut energy_collector_count = 0;
        let mut mineral_collector_count = 0;
        let mut scientist_count = 0;
        let mut exploring_count = 0;
        let mut returning_count = 0;
        let mut at_station_count = 0;
        let mut dead_count = 0;

        let mut total_energy = 0;
        let mut total_minerals = 0;
        let mut total_science = 0;

        for robot in &self.robots {
            // Count by type
            match robot.robot_type {
                RobotType::Explorer => explorer_count += 1,
                RobotType::EnergyCollector => energy_collector_count += 1,
                RobotType::MineralCollector => mineral_collector_count += 1,
                RobotType::Scientist => scientist_count += 1,
            }

            // Count by state (and check for dead robots)
            if robot.energy == 0 {
                dead_count += 1;
            } else {
                match robot.state {
                    crate::robot::RobotState::Exploring => exploring_count += 1,
                    crate::robot::RobotState::ReturningToStation => returning_count += 1,
                    crate::robot::RobotState::AtStation => at_station_count += 1,
                }
            }

            // Sum resources
            total_energy += robot.energy;
            total_minerals += robot.minerals;
            total_science += robot.science_points;
        }

        format!(
            "Swarm: {} robots | Types: E:{} En:{} M:{} S:{} | States: Exploring:{} Returning:{} AtStation:{} Dead:{} | Total Cargo: Energy:{} Minerals:{} Science:{}",
            self.robots.len(),
            explorer_count, energy_collector_count, mineral_collector_count, scientist_count,
            exploring_count, returning_count, at_station_count, dead_count,
            total_energy, total_minerals, total_science
        )
    }
}

// The RobotExplorationUpdate type and known_map field handle shared map information
