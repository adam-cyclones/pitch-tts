#!/bin/bash

# Test script for pitch-tts
# Tests are configured for single-threaded execution in Cargo.toml

echo "🧪 Running pitch-tts tests..."

# Run all tests (single-threaded by default)
cargo test -- --test-threads=1

echo "✅ All tests completed!" 