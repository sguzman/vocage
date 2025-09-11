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

      # Use latest stable available now (swap to ."1.89.0" when the attr exists)
      rustToolchain = pkgs.rust-bin.stable.latest.default;

      naerskLib = pkgs.callPackage naersk {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    in rec {
      packages.vocage = naerskLib.buildPackage {
        pname = "vocage";
        src = ./.;

        # naersk expects a single hash of the vendored deps tree
        cargoHash = lib.fakeSha256; # replace after first prefetch

        nativeBuildInputs = [pkgs.pkg-config];
        cargoBuildOptions = opts: opts ++ ["--verbose"];
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
