{
  description = "Personal web server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override{
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

        # Python environment for Robot Framework
        pythonEnv = pkgs.python3.withPackages (ps: with ps; [
          robotframework
          robotframework-seleniumlibrary
          robotframework-requests
          robotframework-databaselibrary
          requests
        ]);

        # Testing tools
        testTools = with pkgs; [
          chromium
          firefox
          geckodriver
          chromedriver
          curl
          httpie
          procps
          jq
          yq
        ];
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.openssl
            pkgs.jetbrains.rust-rover
            pkgs.code-cursor
            pkgs.sqlite
            pkgs.lldb
            pythonEnv
            testTools
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PYTHONPATH = "${pythonEnv}/${pythonEnv.sitePackages}";

          shellHook = ''
            echo "🚀 Ready to develop with Rocket and rust-analyzer!"
            echo "🧪 Robot Framework and testing tools available!"
            echo "Available commands:"
            echo "  cargo test          - Run Rust unit tests"
            echo "  robot tests/robot/  - Run Robot Framework tests"
            echo "  cargo run           - Start the server"
            echo "  cargo build --release - Build for production"
          '';
        };
      }
    );
}
