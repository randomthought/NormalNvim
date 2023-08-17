# {
#   description = "A Nix-flake-based Rust development environment";
#
#   inputs = {
#     nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
#     rust-overlay.url = "github:oxalica/rust-overlay";
#   };
#
#   outputs = { self, nixpkgs, rust-overlay }:
#     let
#       overlays = [
#         rust-overlay.overlays.default
#         (final: prev: {
#           rustToolchain =
#             let
#               rust = prev.rust-bin;
#             in
#             if builtins.pathExists ./rust-toolchain.toml then
#               rust.fromRustupToolchainFile ./rust-toolchain.toml
#             else if builtins.pathExists ./rust-toolchain then
#               rust.fromRustupToolchainFile ./rust-toolchain
#             else
#               rust.stable.latest.default;
#         })
#       ];
#       supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
#       forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
#         pkgs = import nixpkgs { inherit overlays system; };
#       });
#     in
#     {
#       devShells = forEachSupportedSystem ({ pkgs }: {
#         default = pkgs.mkShell {
#           packages = with pkgs; [
#             rustToolchain
#             openssl
#             pkg-config
#             cargo-deny
#             cargo-edit
#             cargo-watch
#             rust-analyzer
#           ];
#         };
#       });
#     };
# }

{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.treefmt-nix.flakeModule
      ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          nonRustDeps = with pkgs; [
            darwin.apple_sdk.frameworks.Security # Only needed for macos
            pkgconfig 
            openssl 
            libiconv
          ];
        in
        {
          # Rust package
          packages.default = pkgs.rustPlatform.buildRustPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          # Rust dev environment
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.treefmt.build.devShell
            ];
            shellHook = ''
              # For rust-analyzer 'hover' tooltips to work.
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            '';
            buildInputs = nonRustDeps;
            nativeBuildInputs = with pkgs; [
              just
              rustc
              cargo
              cargo-watch
              rust-analyzer
              clang
            ];
          };

          # Add your auto-formatters here.
          # cf. https://numtide.github.io/treefmt/
          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };
        };
    };
}
