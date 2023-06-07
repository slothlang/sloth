{
  description = "slothlang";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustStable = pkgs.rust-bin.stable.latest.default;
        rustNightly = pkgs.rust-bin.nightly."2023-02-10".default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustStable;
          rustc = rustStable;
        };
      in
      with pkgs;
      {
        packages.default = rustPlatform.buildRustPackage rec {
          pname = "sloth";
          version = "0.1.0";
          src = ./.;

          # FIXME: Tests do not run in release mode
          checkType = "debug";
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          meta = with lib; {
            description = "The Sloth programming language";
            homepage = "https://slothlang.tech";
            license = with licenses; [ mit asl20 ];
          };
        };
        devShells.default = mkShell {
          buildInputs = [
            (rustNightly.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })

            cargo-watch
            cargo-deny
            cargo-release

            pkg-config

            # Packages required for LLVM
            llvmPackages_15.libllvm
            libffi
            libxml2

            # C compiler for debugging
            clang
          ];
        };
      }
    );
}
