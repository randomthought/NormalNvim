{
  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/nixpkgs-unstable;
    fenix = {
      url = github:nix-community/fenix;
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, fenix, flake-utils, }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        devToolchain = fenix.packages."${system}".stable;
        nonRustDeps = with pkgs; [
          # darwin.apple_sdk.frameworks.Security # Only needed for macos
          darwin.apple_sdk.frameworks.Foundation
          libiconv
        ];
      in rec {
        devShell = pkgs.mkShell {
          buildInputs = nonRustDeps;
          nativeBuildInputs = [
            (devToolchain.withComponents [
              "cargo"
              "rustc" 
              "rust-src" 
              "rustfmt" 
              "clippy"
            ])
          ];
        };
      });
}
