{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        buildRustPackageWithDocker = { dir, ports ? null, volumes ? null }:
          let
            # Read package meta from Cargo.toml
            meta = (builtins.fromTOML (builtins.readFile ./${dir}/Cargo.toml)).package;

            self = rustPlatform.buildRustPackage rec {
              pname = meta.name;
              version = meta.version;
              src = ./.;
              buildAndTestSubdir = dir;
              cargoLock.lockFile = ./Cargo.lock;
              nativeBuildInputs = [
                pkgs.pkg-config
              ];
              buildInputs = [
                pkgs.openssl
              ];

              passthru.docker = pkgs.dockerTools.buildImage {
                name = meta.name;
                config = {
                  Cmd = [
                    "${self}/bin/${meta.name}"
                  ];
                  ExposedPorts = ports;
                  Volumes = volumes;
                };
              };
            };
          in
          self;

        ciCargoFmt = pkgs.writeScriptBin "ci-cargo-fmt" ''
          echo "🧹 running cargo fmt..."
          cargo fmt --check
        '';

        ciCargoClippy = pkgs.writeScriptBin "ci-cargo-clippy" ''
          echo "🦄 running cargo clippy..."
          cargo clippy
        '';

        mkCiBuildAndTagImage = { target }: pkgs.writeScriptBin "ci-build-${target}" ''
          echo "🐋 building ${target} docker image..."
          nix build .#${target}.docker
          name=$(nix eval --raw .#${target}.docker.imageName)
          tag=$(nix eval --raw .#${target}.docker.imageTag)
          echo "📦 loading image $name:$tag"
          docker load -q -i result
          echo "🚢 image loaded"
        '';

        ciBuildAndTagCli = mkCiBuildAndTagImage { target = "ryogoku"; };
        ciBuildAndTagOperator = mkCiBuildAndTagImage { target = "operator"; };

        ciLocal = pkgs.writeScriptBin "ci-local" ''
          ci-cargo-fmt
          ci-cargo-clippy
          ci-build-ryogoku
          ci-build-operator
        '';
      in
      {
        formatter = pkgs.nixpkgs-fmt;
        packages = {
          ryogoku = buildRustPackageWithDocker {
            dir = "cli";
          };

          operator = buildRustPackageWithDocker {
            dir = "operator";
          };
        };
        devShells.default = pkgs.mkShell {

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          buildInputs = with pkgs; [
            # ci scripts
            ciCargoFmt
            ciCargoClippy
            ciBuildAndTagCli
            ciBuildAndTagOperator
            ciLocal

            # rust with source for lsp
            (rustVersion.override {
              extensions = [ "rust-src" ];
            })

            # dependencies and other useful tools
            clang
            openssl
            kubectl
            minikube
            pkg-config
          ];
        };
      }
    );
}
