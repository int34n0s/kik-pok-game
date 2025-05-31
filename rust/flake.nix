{
  description = "Cross compiling a rust program for windows";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
  };

  outputs = { nixpkgs, crane, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-KUm16pHj+cRedf8vxs/Hd2YWxpOrWZ7UOrwhILdSJBU=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        inherit (pkgs) lib;
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # Example of a folder for images, icons, etc
            (lib.fileset.maybeMissing ./assets)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        reaction-app = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          buildInputs = [
            # Add additional build inputs here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        });

        reaction-app-windows = craneLib.buildPackage (commonArgs // {
          doCheck = false;
          release = false;

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

          # fixes issues related to libring
          TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
            pkgsCross.mingwW64.windows.pthreads
          ];

          CARGO_PROFILE = "dev";
        });
      in
      {
        packages = {
          inherit reaction-app-windows;
          default = reaction-app;
        };

        checks = {
          inherit reaction-app-windows;
        };
      }
    );
}
