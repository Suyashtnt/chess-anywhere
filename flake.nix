{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      rustTC = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      packages = with pkgs; [
        # binaries
        rustTC
        cargo-watch
        cargo-binstall # just in case

        # lsps
        nil
        marksman
        alejandra
      ];
    in {
      devShell = pkgs.mkShell {
        buildInputs = packages;

        PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
        PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
        RUST_SRC_PATH = "${rustTC}/lib/rustlib/src/rust/library/";
      };
    });
}