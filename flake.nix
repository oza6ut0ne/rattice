{
  description = "A media viewer for web browsers.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, ... }:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        { system, ... }:
        let
          name = "rattice";
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };

          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.minimal;
            rustc = pkgs.rust-bin.stable.latest.minimal;
          };
        in
        {
          packages = rec {
            default = bin;

            bin = rustPlatform.buildRustPackage {
              inherit name;
              src = ./.;
              version = self.shortRev or self.dirtyShortRev or "dev";

              cargoLock = {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };

              meta.mainProgram = name;
            };

            docker = pkgs.dockerTools.buildLayeredImage {
              inherit name;
              tag = "latest";
              contents = [ bin ];
              config = {
                WorkingDir = "/workdir";
                Entrypoint = "/bin/rattice";
              };
            };
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs.rust-bin; [
              (stable.latest.minimal.override {
                extensions = [
                  "clippy"
                  "rust-src"
                ];
              })
              nightly.latest.rustfmt
              nightly.latest.rust-analyzer
            ];

            RUST_BACKTRACE = 1;
          };
        };
    };
}
