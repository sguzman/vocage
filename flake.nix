{
  description = "Vocage time-staggered learning (like Anki)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            fenix.overlay
            (final: prev: {
              rustToolchain = fenix.packages.${system}.stable.toolchain;
            })
          ];
        };

        naersk-lib = naersk.lib.${system}.override {
          cargo = pkgs.rustToolchain;
          rustc = pkgs.rustToolchain;
        };

        vocage = naersk-lib.buildPackage {
          pname = "vocage";
          version = "1.1.0";
          src = ./.;
          # You must run `cargo vendor` first!
          cargoVendorDir = "vendor";
        };

      in { packages.default = vocage; });
}

