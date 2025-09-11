{
  description = "Vocage: time-staggered learning (like Anki)";

  inputs = {
    # 25.05 release channel
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

      # NOTE:
      #  - 1.89.0 isn't present; use latest stable for now.
      #  - When 1.89.0 becomes available, change this to:
      #      pkgs.rust-bin.stable."1.89.0".default
      rustToolchain = pkgs.rust-bin.stable.latest.default;

      # Wire naersk to that toolchain
      naerskLib = pkgs.callPackage naersk {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    in rec {
      packages.vocage = naerskLib.buildPackage {
        pname = "vocage";
        src = ./.; # expects Cargo.toml & Cargo.lock here

        # Build release artifacts
        cargoBuildOptions = opts: opts ++ ["--verbose"];

        # Add native deps here if your crates need them (openssl, sqlite, zlib, …)
        nativeBuildInputs = [pkgs.pkg-config];

        # If you have git deps in Cargo.lock, uncomment and fill in hashes:
        # cargoLock = {
        #   lockFile = ./Cargo.lock;
        #   outputHashes = {
        #     "some-git-crate-0.1.2" = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        #   };
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
