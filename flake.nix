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
        engineDockerImage = pkgs.dockerTools.buildImage {
          name = "${engineApp}";
          contents = [ appRustBuild pkgs.cacert ];
          config = { 
            Entrypoint = [ "${appRustBuild}/bin/${engineApp}" ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };

        forwarderApp = "forwarder";
        forwarderDockerImage = pkgs.dockerTools.buildImage {
          name = "${forwarderApp}";
          contents = [ appRustBuild pkgs.cacert ];
          config = { 
            Entrypoint = [ "${appRustBuild}/bin/${forwarderApp}" ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
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
