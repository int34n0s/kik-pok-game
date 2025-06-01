{
  description = "Kik-Pok Game Engine - A Rust library for Godot with multiplayer support";

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

  outputs = { self, nixpkgs, crane, fenix, flake-utils, ... }:
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
          
          buildInputs = [
            # Add additional build inputs here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        };

        kik-pok-engine = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          
          # Since this is a cdylib, we want to build it as a library
          cargoExtraArgs = "--lib";
        });

        kik-pok-engine-windows-debug = craneLib.buildPackage (commonArgs // {
          doCheck = false;
          release = false;

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
          CARGO_TARGET_DIR = "./target/windows";

          # fixes issues related to libring
          TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
            pkgsCross.mingwW64.windows.pthreads
          ];

          CARGO_PROFILE = "dev";
          
          # Since this is a cdylib, we want to build it as a library
          cargoExtraArgs = "--lib";
          
          # Ensure output goes to target/windows/debug
          postInstall = ''
            mkdir -p $out/target/windows/debug
            cp -r ./target/windows/x86_64-pc-windows-gnu/debug/* $out/target/windows/debug/ || true
          '';
        });

        kik-pok-engine-windows-release = craneLib.buildPackage (commonArgs // {
          doCheck = false;
          release = true;

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
          CARGO_TARGET_DIR = "./target/windows";

          # fixes issues related to libring
          TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
            pkgsCross.mingwW64.windows.pthreads
          ];

          CARGO_PROFILE = "release";
          
          # Since this is a cdylib, we want to build it as a library
          cargoExtraArgs = "--lib";
          
          # Ensure output goes to target/windows/release
          postInstall = ''
            mkdir -p $out/target/windows/release
            cp -r ./target/windows/x86_64-pc-windows-gnu/release/* $out/target/windows/release/ || true
          '';
        });
      in
      {
        packages = {
          inherit kik-pok-engine-windows-debug kik-pok-engine-windows-release;
          default = kik-pok-engine;
          windows-debug = kik-pok-engine-windows-debug;
          windows-release = kik-pok-engine-windows-release;
        };

        checks = {
          inherit kik-pok-engine-windows-debug kik-pok-engine-windows-release;
        };
        
        devShells.default = craneLib.devShell {
          # Inherit inputs from checks
          checks = self.checks.${system};
          
          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";
          
          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.godot_4
            pkgs.pkg-config
          ];
        };
      }
    );
}
