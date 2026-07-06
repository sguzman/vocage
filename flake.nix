{
  description = "Rust package built with naersk";

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
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [rust-overlay.overlays.default];
        pkgs = import nixpkgs {inherit system overlays;};
        lib = pkgs.lib;

        # Pull name/version straight from Cargo.toml
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        pname = cargoToml.package.name or "app";
        version = cargoToml.package.version or "0.0.0";

        # Toolchain (use exact pin later if you want)
        rustToolchain = pkgs.rust-bin.stable.latest.default;

        naersk' = pkgs.callPackage naersk {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in rec {
        packages.${pname} = naersk'.buildPackage {
          inherit pname version;
          src = ./.;

          # You said: remove --release; add --verbose
          release = false;
          cargoBuildOptions = opts: opts ++ ["--verbose"];

          # Enforce lockfile + no network inside sandbox
          cargoOptions = opts: opts ++ ["--offline" "--locked"];

          # Add more natives here if needed (openssl, sqlite, zlib, …)
          nativeBuildInputs = [pkgs.pkg-config];
        };

        packages.default = packages.${pname};

        apps.default = {
          type = "app";
          program = "${packages.${pname}}/bin/${pname}";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.pkg-config
          ];
        };
      }
    );
}
