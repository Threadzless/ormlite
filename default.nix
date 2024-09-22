{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [
    nixd
    rustup
    cargo
  ];

  buildInputs = [
    udev
  ];

  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
