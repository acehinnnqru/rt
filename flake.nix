{
  description = "rt: Rust Git repository manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    devshells-rust.url = "github:acehinnnqru/devshells?dir=langs/rust/nightly";
    devshells-nix.url = "github:acehinnnqru/devshells?dir=langs/nix";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      devshells-rust,
      devshells-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustDevShell = devshells-rust.devShells.${system}.default;
        nixDevShell = devshells-nix.devShells.${system}.default;

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
      in
      {
        packages = {
          default = rt;
          rt = rt;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = rt;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            rustDevShell
            nixDevShell
          ];
        };
      }
    );
}
