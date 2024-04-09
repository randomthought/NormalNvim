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
          libiconv
        ] ++ (if stdenv.hostPlatform.system == "x86_64-darwin" then [ darwin.apple_sdk.frameworks.Foundation ] else []);

        rustPlatform = pkgs.makeRustPlatform {
          cargo = devToolchain.cargo;
          rustc = devToolchain.rustc;
        };

        version = "0.1.0";

        appName = "trade_engine";
        appRustBuild = rustPlatform.buildRustPackage {
          pname = "${appName}";
          version = version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = nonRustDeps;
        };

        engineApp = "engine";
        engineDockerImage = let 
          appName = "engine";
        in pkgs.dockerTools.buildImage {
          name = "${appName}";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            pathsToLink = ["/"];
            paths = [ appRustBuild ]; # TODO: ensure you only copy the app binery
          };
          config = { 
            Entrypoint = [ "${appRustBuild}/bin/${appName}" ];
          };
        };

        forwarderDockerImage = let
          appName = "forwarder";
        in pkgs.dockerTools.buildImage {
          name = "${appName}";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            pathsToLink = ["/"];
            paths = [ appRustBuild ]; # TODO: ensure you only copy the app binery
          };
          config = { 
            Entrypoint = [ "${appRustBuild}/bin/${appName}" ];
          };
        };

      in rec {
        packages = {
          rustPackage = appRustBuild;
          engineDocker = engineDockerImage;
          forwarderDocker = forwarderDockerImage;
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
