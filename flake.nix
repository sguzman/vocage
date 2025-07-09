{
  description = "Vocage time-staggered learning (like Anki)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable."1.85.0".default;
      in
      {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "vocage";
          version = "1.1.0";

          src = ./.;

          nativeBuildInputs = [ rustToolchain ];

          buildPhase = ''
            cargo build --release --verbose
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/ $out/bin/
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [ rustToolchain ];
        };
      }
    );
}

