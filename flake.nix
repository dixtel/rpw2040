{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/release-24.05";
      flake-utils.url  = "github:numtide/flake-utils";
      rust-overlay.url = "github:oxalica/rust-overlay";
    };
  
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };   
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [ 
            openssl
            pkg-config
            probe-rs
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" "cargo" "rustfmt" "clippy" ];
              targets = [ "thumbv6m-none-eabi" ];
            })
          ];
        };
      }
    );
}
