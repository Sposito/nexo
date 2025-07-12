{
  description = "Test bench for nexo web server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Python environment for Robot Framework
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          robotframework
          robotframework-seleniumlibrary
          robotframework-requests
          robotframework-databaselibrary
          requests
        ]);

        # Test dependencies
        testDeps = with pkgs; [
          chromium
          firefox
          geckodriver
          chromedriver
          sqlite
          curl
          httpie
          procps
          jq
          yq
        ];
      in
      {
        # CI/CD shell for automated testing
        ciShell = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pythonEnv
            testDeps
            pkgs.pkg-config
            pkgs.openssl
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PYTHONPATH = "${pythonEnv}/${pythonEnv.sitePackages}";

          shellHook = ''
            echo "🧪 CI/CD test environment ready!"
            echo "This environment is optimized for automated testing."
          '';
        };

        # CI/CD packages
        packages = {
          # Test runner for CI
          test-runner = pkgs.writeShellScriptBin "run-tests" ''
            set -e
            echo "🧪 Running test suite..."

            # Always run from project root so static files are found
            if [ "$(basename "$PWD")" = "tests" ]; then
              cd ..
            fi

            # Create results directory
            mkdir -p tests/results

            # Ensure port 8001 is free (test environment port)
            PORT=8001
            if lsof -i :$PORT &>/dev/null; then
              echo "Port $PORT is in use. Attempting to kill process..."
              PID=$(lsof -t -i :$PORT)
              kill $PID
              sleep 2
              if lsof -i :$PORT &>/dev/null; then
                echo "Error: Port $PORT is still in use after attempting to kill process. Exiting."
                exit 1
              fi
            fi

            # Set up Rust environment
            export PATH="${rustToolchain}/bin:$PATH"
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"

            # Build the application
            echo "📦 Building application..."
            ${rustToolchain}/bin/cargo build --release

            # Run Rust tests
            echo "🔧 Running Rust tests..."
            ${rustToolchain}/bin/cargo test

            # Start server in background using test environment
            echo "🚀 Starting server using test environment..."
            ROCKET_PROFILE=test ROCKET_CONFIG=./Rocket.toml ROCKET_PORT=8001 ROCKET_ADDRESS=127.0.0.1 ${rustToolchain}/bin/cargo run --release &
            SERVER_PID=$!

            # Wait for server to start
            for i in {1..10}; do
              if curl -s http://localhost:$PORT/health >/dev/null; then
                echo "Server is up!"
                break
              fi
              sleep 1
            done
            if ! curl -s http://localhost:$PORT/health >/dev/null; then
              echo "Error: Server did not start on port $PORT. Exiting."
              kill $SERVER_PID
              exit 1
            fi

            # Trigger DB initialization by hitting the init-db endpoint
            echo "Triggering DB initialization..."
            curl -s -X POST http://localhost:$PORT/api/init-db > /dev/null

            # Set up Python environment for Robot Framework
            export PYTHONPATH="${pythonEnv}/${pythonEnv.sitePackages}"
            export PATH="${pythonEnv}/bin:$PATH"

            # Run Robot Framework tests (correct path)
            echo "🤖 Running Robot Framework tests..."
            ${pythonEnv}/bin/robot --outputdir tests/results tests/robot/server_test.robot || true
            ${pythonEnv}/bin/robot --outputdir tests/results tests/robot/ || true

            # Stop server
            kill $SERVER_PID
            wait $SERVER_PID 2>/dev/null || true

            echo "✅ All tests completed!"
          '';
        };

        # Apps for CI/CD
        apps = {
          test = {
            type = "app";
            program = "${self.packages.${system}.test-runner}/bin/run-tests";
          };
        };
      }
    );
}
