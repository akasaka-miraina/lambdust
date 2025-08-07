//! Enhanced REPL demonstration and usage example.
//!
//! This example shows how to use the enhanced REPL system with all its features:
//! - Debugger integration
//! - History management with search
//! - Intelligent autocompletion
//! - Syntax highlighting
//! - Session management
//! - Code introspection

use lambdust::{Lambdust, Result};


fn main() -> Result<()> {
    println!("🚀 Lambdust Enhanced REPL Demo");
    println!("===============================");

    // Create a Lambdust instance
    let lambdust = Lambdust::new();
    
    #[cfg(feature = "enhanced-repl")]
    {
        println!("✅ Enhanced REPL features available");
        
        // Create a custom configuration
        let mut config = ReplConfig::default();
        config.syntax_highlighting = true;
        config.auto_completion = true;
        config.debugger_enabled = true;
        config.max_history = 500;
        config.session_management = true;
        config.profiling_enabled = true;
        
        println!("🎯 Configuration:");
        println!("   • Syntax highlighting: {}", config.syntax_highlighting);
        println!("   • Auto-completion: {}", config.auto_completion);  
        println!("   • Debugger: {}", config.debugger_enabled);
        println!("   • History limit: {}", config.max_history);
        println!("   • Session management: {}", config.session_management);
        println!("   • Profiling: {}", config.profiling_enabled);
        println!();
        
        println!("🔧 Available Enhanced REPL Commands:");
        println!("   :help          - Show all commands");
        println!("   :debug step    - Step through code");
        println!("   :debug break   - Set breakpoints");
        println!("   :history       - Show command history");
        println!("   :inspect <fn>  - Inspect functions");
        println!("   :session save  - Save current session");
        println!("   :apropos <term> - Search documentation");
        println!();
        
        println!("🎨 Enhanced Features:");
        println!("   • Multi-line input with proper indentation");
        println!("   • Bracket matching and auto-completion");
        println!("   • Context-aware completions");
        println!("   • File path completion for load operations");
        println!("   • Reverse history search (Ctrl+R)");
        println!("   • Session replay and export");
        println!();
        
        // Create and start the enhanced REPL
        let mut repl = EnhancedRepl::new(lambdust, config)?;
        
        println!("🚀 Starting Enhanced REPL...");
        println!("   Type (exit) to quit, :help for commands");
        println!();
        
        repl.run()
    }
    
    #[cfg(all(feature = "repl", not(feature = "enhanced-repl")))]
    {
        println!("⚠️  Enhanced REPL features not available");
        println!("   To enable: cargo run --features enhanced-repl --example enhanced_repl_demo");
        println!();
        println!("   Available features with enhanced-repl:");
        println!("   • Advanced syntax highlighting");
        println!("   • Intelligent autocompletion");
        println!("   • Step-through debugging");
        println!("   • Persistent session management");
        println!("   • Code introspection tools");
        println!("   • Multi-line input with auto-indentation");
        Ok(())
    }
    
    #[cfg(not(any(feature = "repl", feature = "enhanced-repl")))]
    {
        println!("❌ No REPL features available");
        println!("   To enable basic REPL: cargo run --features repl --example enhanced_repl_demo");
        println!("   To enable enhanced REPL: cargo run --features enhanced-repl --example enhanced_repl_demo");
        Ok(())
    }
}

#[cfg(feature = "enhanced-repl")]
fn demonstrate_features() {
    println!("🎯 Enhanced REPL Feature Showcase:");
    println!();
    
    println!("1. 🐛 Debugger Integration:");
    println!("   :debug enable         - Enable debugging");
    println!("   :debug break (+ 1 2)  - Set breakpoint on expression");
    println!("   :debug step           - Step through execution");
    println!("   :debug continue       - Continue execution");
    println!("   :debug stack          - Show call stack");
    println!("   :debug vars           - Show variables");
    println!();
    
    println!("2. 📚 History Management:");
    println!("   :history              - Show recent commands");
    println!("   :history search map   - Search for 'map' in history");
    println!("   :history clear        - Clear history");
    println!("   Ctrl+R                - Reverse history search");
    println!();
    
    println!("3. 💡 Intelligent Completion:");
    println!("   (m<TAB>              - Complete to 'map', 'max', etc.");
    println!("   (string-<TAB>        - Show string functions");
    println!("   (import <TAB>        - Show available libraries");
    println!("   \"path/<TAB>          - Complete file paths");
    println!();
    
    println!("4. 🔍 Code Introspection:");
    println!("   :inspect map         - Show function documentation");
    println!("   :describe car        - Detailed function description");
    println!("   :apropos list        - Find functions related to 'list'");
    println!("   :source my-function  - Show function source (if available)");
    println!();
    
    println!("5. 💾 Session Management:");
    println!("   :session save my-work    - Save current session");
    println!("   :session load my-work    - Load a session");
    println!("   :session list            - List all sessions");
    println!("   :session replay          - Replay session commands");
    println!();
    
    println!("6. 🎨 Enhanced UX:");
    println!("   • Syntax highlighting for all Scheme constructs");
    println!("   • Bracket matching with visual indicators");
    println!("   • Multi-line input with smart indentation");
    println!("   • Error recovery and helpful suggestions");
    println!("   • Configurable color schemes");
    println!();
    
    println!("Example multi-line input:");
    println!("   λust:1> (define (factorial n)");
    println!("   λust:...   (if (= n 0)");
    println!("   λust:...       1");  
    println!("   λust:...       (* n (factorial (- n 1)))))");
    println!("   => factorial");
    println!();
    
    println!("Example debugging session:");
    println!("   λust:2> :debug break factorial");
    println!("   Breakpoint set on: factorial");
    println!("   λust:3> (factorial 5)");
    println!("   🔴 Breakpoint hit: factorial");
    println!("   λust:debug:4> :debug step");
    println!("   Stepping into next expression...");
    println!();
}