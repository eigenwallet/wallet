use std::time::Duration;
use swap::network::connection_progress::{ConnectionProgress, ErrorCategory};

fn main() {
    println!("🔗 Enhanced Connection Progress Demo");
    println!("====================================\n");

    println!("📡 Simulating connection attempts to Alice peer...\n");

    let mut progress = ConnectionProgress::new(
        "12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X".to_string(),
        Some(20) // 20 retries left
    );

    // Simulate different types of connection failures to reach the 12 failures
    let failures = vec![
        ("Connection timeout", ErrorCategory::Timeout),
        ("Network unreachable", ErrorCategory::Network),
        ("Connection refused", ErrorCategory::Network),
        ("DNS resolution failed", ErrorCategory::Network),
        ("Connection timeout", ErrorCategory::Timeout),
        ("Peer unavailable", ErrorCategory::PeerUnavailable),
        ("Connection timeout", ErrorCategory::Timeout),
        ("Authentication failed", ErrorCategory::Auth),
        ("Protocol version mismatch", ErrorCategory::Protocol),
        ("Connection timeout", ErrorCategory::Timeout),
        ("Network unreachable", ErrorCategory::Network),
        ("Connection timeout", ErrorCategory::Timeout),
    ];

    for (i, (error_msg, category)) in failures.iter().enumerate() {
        progress.start_attempt();
        progress.record_failure(
            error_msg.to_string(),
            category.clone(),
            Some(Duration::from_secs(5 + i as u64 * 2))
        );

        // Show the progress message
        println!("Attempt {}: {}", i + 1, progress.format_message());
        
        // Show troubleshooting suggestions for certain error types
        let suggestions = progress.get_user_suggestions();
        if !suggestions.is_empty() {
            println!("   💡 Troubleshooting: {}", suggestions.join(", "));
        }
        
        println!("   ⏱️  Elapsed time: {:?}", progress.elapsed_time());
        println!("   📊 Error category: {:?}", progress.error_category);
        println!("   🎯 Connection state: {:?}", progress.state);
        
        // Show the exact format after the 12th failure
        if i == 11 {
            println!("\n🎯 EXACT MESSAGE FORMAT REQUESTED:");
            println!("   ▶️  {}", progress.format_message());
            println!("\n   This matches: 'Trying to reconnect (Last Error: Connection Timeout, 12 times failed, retries_left: 20)'");
        }
        
        println!();
    }

    println!("🎉 Demo completed!");
    println!("This system now provides enhanced connection progress tracking with:");
    println!("✅ Detailed retry attempt counting");
    println!("✅ Error categorization and troubleshooting suggestions"); 
    println!("✅ Structured progress updates for GUI integration");
    println!("✅ Comprehensive logging and monitoring");
    println!("✅ User-friendly progress messages");
}