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
        rustNightly = pkgs.rust-bin.nightly."2023-06-19".default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustStable;
          rustc = rustStable;
        };
      in
      let
        baseNativeBuildInputs = with pkgs; [ pkg-config ];
        baseBuildInputs = with pkgs; [ 
          llvmPackages_15.libllvm
          libffi
          libxml2
        ];
      in
      with pkgs;
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = "sloth";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = baseNativeBuildInputs;
          buildInputs = baseBuildInputs;

          meta = with lib; {
            description = "The Sloth programming language";
            homepage = "https://slothlang.tech";
            license = with licenses; [ mit asl20 ];
          };

          LLVM_SYS_150_PREFIX = "${llvmPackages_15.libllvm.dev}";
        };
        devShells.default = mkShell {
          nativeBuildInputs = baseNativeBuildInputs;
          buildInputs = baseBuildInputs ++ [
            (rustNightly.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })

            cargo-watch
            cargo-deny
            cargo-release

            # C compiler for debugging
            clang
          ];

          RUST_BACKTRACE = 1;
        };
      }
    );
}
