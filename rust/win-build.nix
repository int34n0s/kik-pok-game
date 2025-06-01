# Single command Windows build
# Usage: nix-shell win-build.nix --run "cargo build"
# Or:    nix-shell win-build.nix --run "cargo build --release"

let
  pkgs = import <nixpkgs> {
    overlays = [
      (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
    ];
  };
in pkgs.mkShell {
  buildInputs = with pkgs; [
    (rust-bin.stable.latest.default.override {
      targets = [ "x86_64-pc-windows-gnu" ];
    })
    pkgsCross.mingwW64.stdenv.cc
    pkgsCross.mingwW64.windows.pthreads
  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.libiconv
  ];
  
  CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
  TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";
} 