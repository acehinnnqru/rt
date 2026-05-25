{
  description = "rt: Rust Git repository manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devshells = {
      url = "github:acehinnnqru/devshells";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    devshells,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};

        rt = pkgs.rustPlatform.buildRustPackage {
          pname = "rt";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          meta = with pkgs.lib; {
            description = "Git repository manager with predictable directory tree";
            homepage = "https://github.com/acehinnnqru/rt";
            license = licenses.mit;
            platforms = platforms.unix;
            mainProgram = "rt";
          };
        };
      in {
        packages = {
          default = rt;
          rt = rt;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = rt;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            devshells.devShells.${system}."rust-nightly"
            devshells.devShells.${system}.default
          ];
        };
      }
    );
}
