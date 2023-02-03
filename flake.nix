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

        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        buildRustPackageWithDocker = { crate, dir, ports ? null, volumes ? null }:
          let
            self = rustPlatform.buildRustPackage rec {
              pname = crate;
              version = "0.0.0";
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
                name = crate;
                config = {
                  Cmd = [
                    "${self}/bin/${crate}"
                  ];
                  ExposedPorts = ports;
                  Volumes = volumes;
                };
              };
            };
          in
          self;

        ciCargoFmt = pkgs.writeScriptBin "ci-cargo-fmt" ''
          cargo fmt --check
        '';

        ciCargoClippy = pkgs.writeScriptBin "ci-cargo-clippy" ''
          cargo clippy
        '';

        ciLocal = pkgs.writeScriptBin "ci-local" ''
          echo "Running cargo fmt..."
          ci-cargo-fmt

          echo "Running cargo clippy..."
          ci-cargo-clippy
        '';
      in
      {
        formatter = pkgs.nixpkgs-fmt;
        packages = {
          ryogoku = buildRustPackageWithDocker {
            crate = "ryogoku";
            dir = "cli";
          };

          operator = buildRustPackageWithDocker {
            crate = "ryogoku-operator";
            dir = "operator";
          };
        };
        devShells.default = pkgs.mkShell {

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          buildInputs = with pkgs; [
            # ci scripts
            ciCargoFmt
            ciCargoClippy
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
