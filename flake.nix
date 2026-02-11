{
    description = "TUI for managing NixOS containers";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs, flake-utils }:
        flake-utils.lib.eachDefaultSystem (system: let
            pkgs = nixpkgs.legacyPackages.${system};
            cargo_manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in {
            defaultPackage = pkgs.rustPlatform.buildRustPackage {
                pname = cargo_manifest.package.name;
                version = cargo_manifest.package.version;
                src = ./.;
                cargoLock.lockFile = ./Cargo.lock;
            };
            devShells.default = pkgs.mkShell {
                packages = with pkgs; [
                    cargo
                    clippy
                    rustfmt
                ];
            };
        });
}
