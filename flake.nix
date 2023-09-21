{
  description = "axum-example";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    let
      overlays = [ (import rust-overlay) ];
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs; rec {
        devShell = mkShell rec {
          name = "axum-example";

          nativeBuildInputs = [ pkg-config ];

          buildInputs = [
            ((rust-bin.selectLatestNightlyWith
              (toolchain: toolchain.default)).override {
              targets = [ "wasm32-wasi" ];
            })
            rust-analyzer
          ];

          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      }
    );
}
