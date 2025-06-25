use std::io::{self, Write};

pub struct StartupScreen;

impl StartupScreen {
    pub fn show() -> bool {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Display ASCII art title
        println!("\x1B[95m"); // Magenta color
        println!("██████╗ ██╗   ██╗███████╗████████╗██╗   ██╗");
        println!("██╔══██╗██║   ██║██╔════╝╚══██╔══╝╚██╗ ██╔╝");
        println!("██████╔╝██║   ██║███████╗   ██║    ╚████╔╝ ");
        println!("██╔══██╗██║   ██║╚════██║   ██║     ╚██╔╝  ");
        println!("██║  ██║╚██████╔╝███████║   ██║      ██║   ");
        println!("╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝      ╚═╝   ");
        println!();
        println!("\x1B[96m"); // Cyan color
        println!("███████╗██╗    ██╗ █████╗ ██████╗ ███╗   ███╗");
        println!("██╔════╝██║    ██║██╔══██╗██╔══██╗████╗ ████║");
        println!("███████╗██║ █╗ ██║███████║██████╔╝██╔████╔██║");
        println!("╚════██║██║███╗██║██╔══██║██╔══██╗██║╚██╔╝██║");
        println!("███████║╚███╔███╔╝██║  ██║██║  ██║██║ ╚═╝ ██║");
        println!("╚══════╝ ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝");
        println!("\x1B[0m"); // Reset color

        println!();
        println!("\x1B[93m🤖 Autonomous Robot Exploration Simulation 🤖\x1B[0m");
        println!();
        println!("\x1B[92mFeatures:\x1B[0m");
        println!("  🔍 Intelligent swarm exploration");
        println!("  ⚡ Resource collection & management");
        println!("  🏭 Autonomous robot creation");
        println!("  🗺️  Real-time map discovery");
        println!();
        println!("\x1B[94mControls:\x1B[0m");
        println!("  • Robots explore automatically");
        println!("  • Press 'Q' during game to quit");
        println!();
        println!("\x1B[96m─────────────────────────────────────────────\x1B[0m");
        println!();
        print!("\x1B[95m⚡ Press ENTER to start exploration... \x1B[0m");
        io::stdout().flush().unwrap();

        // Wait for Enter key
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
