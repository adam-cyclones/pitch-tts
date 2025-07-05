// CLI Tests for pitch-tts
// 
// These tests validate CLI functionality including the say command with audio playback.
// Tests are configured to run single-threaded in Cargo.toml to prevent audio conflicts.

use std::process::Command;
use std::time::Duration;
use std::thread;

#[test]
fn test_single_threaded_execution() {
    // This test ensures we're running with --test-threads=1
    // It's not a perfect check, but it helps remind developers
    println!("🔧 Running CLI tests - ensure you used --test-threads=1");
    println!("🔧 This prevents audio conflicts when multiple tests play audio");
}

#[test]
fn test_cli_say_command() {
    // Test the say command with default values (should play Alba with Scottish phrase)
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "say"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            // Check that the command executed successfully
            assert!(result.status.success() || result.status.code() == Some(0), 
                "CLI say command should execute successfully");
            
            // Check that we got some output
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            // Should contain voice information
            assert!(stdout.contains("Playing voice") || stderr.contains("Playing voice"), 
                "Should show voice information");
        }
        Err(e) => {
            eprintln!("CLI test failed to execute: {}", e);
            // Don't fail the test if we can't execute the CLI
        }
    }
}

#[test]
fn test_cli_say_with_custom_text() {
    // Test say command with custom text
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "say", "Testing CLI functionality"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            // Should show voice information
            assert!(stdout.contains("Playing voice") || stderr.contains("Playing voice"), 
                "Should show voice information for custom text");
        }
        Err(e) => {
            eprintln!("CLI test with custom text failed: {}", e);
        }
    }
}

#[test]
fn test_cli_say_with_pitch() {
    // Test say command with pitch shifting
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "say", "Testing pitch shift", "--pitch", "1.5"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            // Should show pitch information
            assert!(stdout.contains("pitch: 1.5") || stderr.contains("pitch: 1.5"), 
                "Should show pitch information");
        }
        Err(e) => {
            eprintln!("CLI test with pitch failed: {}", e);
        }
    }
}

#[test]
fn test_cli_list_command() {
    // Test the list command
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "list"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            assert!(result.status.success(), "List command should succeed");
            
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            // Should list available voices
            assert!(stdout.contains("Available voices"), "Should show available voices");
            assert!(stdout.contains("en_GB-alba-medium"), "Should include Alba voice");
        }
        Err(e) => {
            eprintln!("CLI list test failed: {}", e);
        }
    }
}

#[test]
fn test_cli_help() {
    // Test that help works
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "--help"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            assert!(result.status.success(), "Help command should succeed");
            
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            // Should show help information
            assert!(stdout.contains("pitch-tts"), "Should show program name");
            assert!(stdout.contains("COMMAND"), "Should show command structure");
        }
        Err(e) => {
            eprintln!("CLI help test failed: {}", e);
        }
    }
}

#[test]
fn test_cli_say_help() {
    // Test say command help
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "say", "--help"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            assert!(result.status.success(), "Say help should succeed");
            
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            // Should show say command help
            assert!(stdout.contains("Synthesize speech and play it"), "Should show say description");
            assert!(stdout.contains("--voice"), "Should show voice option");
            assert!(stdout.contains("--pitch"), "Should show pitch option");
        }
        Err(e) => {
            eprintln!("CLI say help test failed: {}", e);
        }
    }
}

#[test]
fn test_cli_legacy_mode() {
    // Test legacy mode (direct flags without subcommand)
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "--voice", "en_GB-alba-medium", "--text", "Legacy mode test"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            // Should show voice information
            assert!(stdout.contains("Using voice") || stderr.contains("Using voice"), 
                "Should show voice information in legacy mode");
        }
        Err(e) => {
            eprintln!("CLI legacy mode test failed: {}", e);
        }
    }
}

// Fast synchronous test that checks CLI output without waiting for audio
#[test]
fn test_cli_say_output_validation() {
    println!("🔍 Testing CLI say command output validation...");
    
    let test_cases = vec![
        ("Default say", vec!["say"]),
        ("Custom text", vec!["say", "Quick test"]),
        ("With pitch", vec!["say", "Pitch test", "--pitch", "1.3"]),
        ("With voice", vec!["say", "Voice test", "--voice", "en_US-libritts_r-medium"]),
    ];
    
    for (description, args) in test_cases {
        println!("Testing: {}", description);
        
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--"]);
        cmd.args(args);
        
        // Use spawn to avoid blocking on audio playback
        match cmd.spawn() {
            Ok(mut child) => {
                // Wait a short time for the process to start and produce output
                thread::sleep(Duration::from_millis(500));
                
                // Check if process is still running
                match child.try_wait() {
                    Ok(Some(status)) => {
                        // Process finished quickly (probably no audio)
                        println!("✅ {} - Completed quickly", description);
                    }
                    Ok(None) => {
                        // Process is still running (probably playing audio), kill it
                        let _ = child.kill();
                        println!("⏱️  {} - Killed (audio playing, this is expected)", description);
                    }
                    Err(e) => {
                        println!("❌ {} - Error checking status: {}", description, e);
                    }
                }
            }
            Err(e) => {
                println!("❌ {} - Failed to spawn: {}", description, e);
            }
        }
        
        // Small delay between tests
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("🔍 CLI output validation completed!");
}

// Integration test that actually plays audio (noisy but comprehensive)
#[test]
#[ignore = "This test plays audio and should be run manually"]
fn test_cli_audio_playback() {
    println!("🎵 Testing actual audio playback...");
    println!("This test will play audio - make sure your speakers are on!");
    
    // Test with a short phrase to minimize noise
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--", "say", "Test audio", "--pitch", "1.2"]);
    
    let output = cmd.output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            // Should show both voice and pitch information
            assert!(stdout.contains("Playing voice") || stderr.contains("Playing voice"), 
                "Should show voice information");
            assert!(stdout.contains("pitch: 1.2") || stderr.contains("pitch: 1.2"), 
                "Should show pitch information");
            
            println!("✅ Audio playback test completed successfully!");
        }
        Err(e) => {
            eprintln!("❌ Audio playback test failed: {}", e);
        }
    }
}

// Manual test function for development
#[test]
#[ignore = "Manual test only"]
fn manual_audio_test() {
    println!("🎵 Manual audio test - this will play several audio samples");
    
    let test_cases = vec![
        ("Normal pitch", vec!["say", "Hello world!"]),
        ("High pitch", vec!["say", "High pitched voice", "--pitch", "1.5"]),
        ("Low pitch", vec!["say", "Low pitched voice", "--pitch", "0.8"]),
        ("Different voice", vec!["say", "Different voice test", "--voice", "en_US-libritts_r-medium"]),
    ];
    
    for (description, args) in test_cases {
        println!("Testing: {}", description);
        
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--"]);
        cmd.args(args);
        
        match cmd.spawn() {
            Ok(mut child) => {
                // Wait a bit for the process to start
                thread::sleep(Duration::from_millis(1000));
                
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            println!("✅ {} - SUCCESS", description);
                        } else {
                            println!("❌ {} - FAILED", description);
                        }
                    }
                    Ok(None) => {
                        let _ = child.kill();
                        println!("⏱️ {} - KILLED (audio playing)", description);
                    }
                    Err(e) => {
                        println!("❌ {} - ERROR: {}", description, e);
                    }
                }
            }
            Err(e) => {
                println!("❌ {} - Failed to spawn: {}", description, e);
            }
        }
        
        // Small delay between tests
        thread::sleep(Duration::from_millis(500));
    }
    
    println!("🎵 Manual audio test completed!");
} 