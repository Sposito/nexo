#!/usr/bin/env bash

# Run tests using the tests flake
# This script allows you to run the CI/CD test suite from the root directory

set -e

echo "🧪 Running test suite using tests flake..."

# Change to tests directory and run the test app
cd tests
nix run .#test

echo "✅ Test suite completed!" 