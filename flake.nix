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

        rustPlatform = pkgs.makeRustPlatform {
          cargo = devToolchain.cargo;
          rustc = devToolchain.rustc;
        };

        version = "0.1.0";

        appName = "trade_engine";
        engineAppRustBuild = rustPlatform.buildRustPackage {
          pname = "${appName}";
          version = version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        engineApp = "engine";
        engineDockerImage = pkgs.dockerTools.buildImage {
          name = "${engineApp}";
          config = { Entrypoint = [ "${appName}/bin/${engineApp}" ]; };
        };

        dataForwarderApp = "data_forwarder";
        dataForwarderDockerImage = pkgs.dockerTools.buildImage {
          name = "${dataForwarderApp}";
          config = { Entrypoint = [ "${appName}/bin/${dataForwarderApp}" ]; };
        };

      in rec {
        packages = {
          rustPackage = appRustBuild;
          engineDocker = engineDockerImage;
          dataForwarderDocker = dataForwarderDockerImage;
        };
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
