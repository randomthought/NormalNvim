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

        appEngineRustBuild = let 
          appName = "engine";
        in rustPlatform.buildRustPackage {
          pname = appName;
          version = version;
          src = ./.;
          buildAndTestSubdir = appName;
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
            pathsToLink = [ "/bin" ];
            paths = [ appEngineRustBuild ];
          };
          config = { 
            Entrypoint = [ "/bin/${appName}" ];
          };
        };

        appForwarderRustBuild = let 
          appName = "forwarder";
        in rustPlatform.buildRustPackage {
          pname = appName;
          version = version;
          src = ./.;
          buildAndTestSubdir = appName;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = nonRustDeps;
        };

        forwarderDockerImage = let
          appName = "forwarder";
        in pkgs.dockerTools.buildImage {
          name = "${appName}";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            pathsToLink = [ "/bin" ];
            paths = [ appForwarderRustBuild ];
          };
          config = { 
            Entrypoint = [ "/bin/${appName}" ];
          };
        };

      in rec {
        packages = {
          rustEnginePackage = appEngineRustBuild;
          rustForwarderPackage = appForwarderRustBuild;
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
