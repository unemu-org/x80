{
  description = "x80 — guest-agnostic Intel 8080-family interpreter library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Backlog.md owns docs and decisions only; beads is installed separately.
    # Its bun2nix build requires its own pinned nixpkgs: do not add follows.
    backlog-md.url = "github:MrLesk/Backlog.md";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, backlog-md, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        # Match x86jit's toolchain; flake.lock preserves the same input pins.
        rustToolchain = pkgs.rust-bin.stable."1.98.1".default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.cargo-nextest
            pkgs.cargo-deny
            pkgs.pre-commit
            backlog-md.packages.${system}.default
          ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          shellHook = ''
            echo "x80 dev shell — rust $(rustc --version)"
          '';
        };
        formatter = pkgs.nixpkgs-fmt;
      });
}
