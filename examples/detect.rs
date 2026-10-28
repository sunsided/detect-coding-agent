//! Example: detect the current AI coding agent environment.
//!
//! Run with:
//! ```sh
//! cargo run --example detect
//! ```

fn main() {
    let agents = detect_coding_agent::detect_all();

    let Some(agent) = agents.first() else {
        println!("❌ No coding agent detected.");
        println!("   The process does not appear to be running inside a known AI coding agent.");
        return;
    };

    println!("✅ Coding agent detected!");
    println!("   Name       : {}", agent.name);
    println!("   ID         : {}", agent.id);
    println!("   Kind       : {}", agent.kind);
    println!("   Confidence : {}", agent.confidence);
    println!("   Signal     : {}", agent.signal);
    if let Some(version) = &agent.version {
        println!("   Version    : {version}");
    }
    if let Some(mode) = &agent.mode {
        println!("   Mode       : {mode}");
    }
    if let Some(ai_agent) = &agent.ai_agent {
        println!("   AI_AGENT   : {ai_agent}");
    }

    if agents.len() > 1 {
        println!();
        println!("   Also matched:");
        for other in &agents[1..] {
            println!(
                "   - {} [{}, {} confidence via {}]",
                other.name, other.kind, other.confidence, other.signal
            );
        }
    }

    if agent.is_agent() || agent.is_hybrid() {
        println!();
        println!("💡 Tip: This is an autonomous agent session — consider offering a");
        println!("   machine-friendly interface (e.g. via MCP) for better results.");
    }
}
