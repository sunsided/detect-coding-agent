//! Example: detect the current AI coding agent environment.
//!
//! Run with:
//! ```sh
//! cargo run --example detect
//! ```

fn main() {
    let agent = detect_coding_agent::detect();

    match &agent {
        Some(agent) => {
            println!("✅ Coding agent detected!");
            println!("   Name : {}", agent.name);
            println!("   ID   : {}", agent.id);
            println!("   Kind : {}", agent.kind);
        }
        None => {
            println!("❌ No coding agent detected.");
            println!(
                "   The process does not appear to be running inside a known AI coding agent."
            );
        }
    }

    // Convenience helpers
    if agent
        .map(|a| a.is_agent() || a.is_hybrid())
        .unwrap_or(false)
    {
        println!();
        println!("💡 Tip: This is an autonomous agent session — consider offering a");
        println!("   machine-friendly interface (e.g. via MCP) for better results.");
    }
}
