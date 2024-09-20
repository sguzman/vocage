{ pkgs ? import <nixpkgs> {} }:

let
  rustPlatform = pkgs.rustPlatform;
  stdenv = pkgs.stdenv;
  fetchFromGitHub = pkgs.fetchFromGitHub;
  Security = pkgs.Security;
  lib = pkgs.lib;
in
rustPlatform.buildRustPackage rec {
  pname = "vocage";
  version = "v1.1.0";

  src = fetchFromGitHub {
    owner = "proycon";
    repo = "vocage";
    rev = version;
    sha256 = "mKMP/ElSipgeAlactNwqE4vCKvqVsemmu99gpi4CWiY=";
  };

  cargoSha256 = "sha256-g1Yg93v+i+zddbvpK/soI8aAgNh6v9HYCXpyujZzrvk=";

  #buildInputs = [ lib.optional stdenv.isDarwin ];

  meta = with lib; {
    description = "A minimalistic spaced-repetion vocabulary trainer (flashcards) for the terminal";
    mainProgram = "vocage";
    homepage = "https://github.com/proycon/vocage";
    license = with licenses; [ unlicense /* or */ mit ];
    maintainers = [ maintainers.proycon ];
  };
}
