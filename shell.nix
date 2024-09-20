with import <nixpkgs> {};

mkShell {
  buildInputs = [
    rustPlatform.rustc
    rustPlatform.cargo
    stdenv
  ];

  RUST_BACKTRACE = "1"; # Useful for debugging rust build issues

  src = ./default.nix;
}

