{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    inputs@{
      flake-parts,
      flake-utils,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = flake-utils.lib.defaultSystems;
      perSystem = { pkgs, ... }:
        {
          packages = {
            default = (pkgs.rustPlatform.buildRustPackage {
              name = "nu_plugin_ws";
              version = (
                builtins.fromTOML (builtins.readFile ./Cargo.toml)
              ).package.version;
              src = ./.;
              nativeBuildInputs = [
                pkgs.pkg-config
              ] ++ pkgs.lib.optionals pkgs.stdenv.cc.isClang [
                pkgs.rustPlatform.bindgenHook
              ];
              buildInputs = [pkgs.openssl.dev pkgs.openssl];
              cargoLock = {lockFile = ./Cargo.lock;};
            });
          };
        };
    };
}
