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

      # use latest stable toolchain available in the overlay
      # (swap to pkgs.rust-bin.stable."1.89.0".default when that attr exists)
      rustToolchain = pkgs.rust-bin.stable.latest.default;

      naerskLib = pkgs.callPackage naersk {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    in rec {
      packages.vocage = naerskLib.buildPackage {
        pname = "vocage";
        src = ./.;

        # make build sandbox-pure/offline
        cargoLock = {lockFile = ./Cargo.lock;};
        cargoHash = lib.fakeSha256; # replace with the real hash after first build

        nativeBuildInputs = [pkgs.pkg-config];

        # your request: remove --release, add --verbose
        cargoBuildOptions = opts: opts ++ ["--verbose"];

        # git deps? uncomment and fill these:
        # cargoLock.outputHashes = {
        #   "crate-or-git-src-and-version" = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
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
