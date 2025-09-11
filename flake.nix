{
  description = "Vocage: time-staggered learning (like Anki)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [rust-overlay.overlays.default];
      pkgs = import nixpkgs {inherit system overlays;};
      lib = pkgs.lib;

      # Pin a toolchain available today (swap to ."1.89.0" once that attr exists)
      rustToolchain = pkgs.rust-bin.stable.latest.default;

      naerskLib = pkgs.callPackage naersk {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    in rec {
      packages.vocage = naerskLib.buildPackage {
        pname = "vocage";
        src = ./.;

        # Make the build fully offline:
        cargoLock = {lockFile = ./Cargo.lock;};

        # STEP 1: start with a fake hash; build once to learn the real one,
        # then replace this value with what Nix prints.
        cargoHash = lib.fakeSha256;

        # Add native deps here if your crates need them (openssl, sqlite, zlib, …)
        nativeBuildInputs = [pkgs.pkg-config];

        cargoBuildOptions = opts: opts ++ ["--release"];

        # If you have git dependencies in Cargo.lock, add them here:
        # cargoLock.outputHashes = {
        #   "crate-name-0.1.2" = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        # };
      };

      packages.default = packages.vocage;

      apps.default = {
        type = "app";
        program = "${packages.vocage}/bin/vocage";
      };

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = [
          rustToolchain
          pkgs.pkg-config
        ];
      };
    });
}
