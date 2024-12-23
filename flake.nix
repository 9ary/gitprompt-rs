{
  description = "A very simple Git prompt written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: let
    inherit (inputs.nixpkgs) lib;
    defaultSystems = lib.systems.flakeExposed;
    argsForSystem = system: {
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [
          (import inputs.rust-overlay)
        ];
        config = {};
      };
    };
    allArgs = lib.genAttrs defaultSystems argsForSystem;
    eachSystem = fn: lib.genAttrs defaultSystems (system: fn allArgs."${system}");

    gitpromptPkg = {
      lib,
      callPackage,
      stdenv,
      pkg-config,
      installShellFiles,
      udev,
    }: let
      naersk = callPackage inputs.naersk {};
    in
      naersk.buildPackage {
        src = ./.;
      };
  in {
    formatter = eachSystem ({pkgs, ...}:
      pkgs.writeShellScriptBin "formatter" ''
        ${pkgs.alejandra}/bin/alejandra flake.nix
      '');

    devShells = eachSystem ({pkgs, ...}: {
      default = pkgs.mkShell {
        name = "gitprompt-rs";
        inputsFrom = [inputs.self.packages."${pkgs.system}".default];
        nativeBuildInputs = [
          (pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-analyzer"
              "rust-src"
            ];
          })
        ];
      };
    });

    packages = eachSystem ({pkgs, ...}: {
      default = pkgs.callPackage gitpromptPkg {};
    });

    overlays.default = final: prev: {
      gitprompt-rs = final.callPackage gitpromptPkg {};
    };
  };
}
