//! Example: detect the current AI coding agent environment.
//!
//! Run with:
//! ```sh
//! cargo run --example detect
//! ```

fn main() {
    match detect_coding_agent::detect() {
        Some(agent) => {
            println!("✅ Coding agent detected!");
            println!("   Name : {}", agent.name);
            println!("   ID   : {}", agent.id);
            println!("   Kind : {}", agent.kind);
        }
        None => {
            println!("❌ No coding agent detected.");
            println!("   The process does not appear to be running inside a known AI coding agent.");
        }
    }

    // Convenience helpers
    if detect_coding_agent::is_agent() {
        println!();
        println!("💡 Tip: This is an autonomous agent session — consider offering a");
        println!("   machine-friendly interface (e.g. via MCP) for better results.");
    }
}
