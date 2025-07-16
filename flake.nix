{
  description = "A media viewer for web browsers.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nix-appimage = {
      url = "github:ralismark/nix-appimage";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, ... }:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } rec {
      systems = [
        "x86_64-linux"
        "armv7l-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        { system, lib, ... }:
        let
          name = "rattice";
          pname = name;
          crossSystems = systems ++ [
            "mipsel-linux-gnu"
          ];

          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };

          pkgsCrossFor =
            crossSystem:
            import inputs.nixpkgs {
              inherit system overlays crossSystem;
            };

          rustPlatformFor =
            pkgs:
            if pkgs.system == "x86_64-linux" then
              pkgs.makeRustPlatform {
                cargo = pkgs.rust-bin.stable.latest.minimal;
                rustc = pkgs.rust-bin.stable.latest.minimal;
              }
            else
              pkgs.rustPlatform;

          buildFor =
            pkgs:
            (rustPlatformFor pkgs).buildRustPackage {
              inherit name pname;
              src = ./.;
              version = self.shortRev or self.dirtyShortRev or "dev";
              meta.mainProgram = name;

              cargoLock = {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };
            };

          buildDockerFor =
            pkgs:
            pkgs.dockerTools.buildLayeredImage {
              inherit name;
              tag = "latest";
              contents = [ (buildFor pkgs) ];
              config = {
                WorkingDir = "/workdir";
                Entrypoint = "/bin/${name}";
              };
            };

          buildAppImageFor =
            package:
            inputs.nix-appimage.lib.${system}.mkAppImage {
              program = "${package}/bin/${name}";
            };
        in
        {
          packages = rec {
            default = bin;
            bin = buildFor pkgs;
            static = buildFor pkgs.pkgsStatic;
            docker = buildDockerFor pkgs;
            appimage = buildAppImageFor bin;
          };

          legacyPackages =
            # Cross build
            (lib.listToAttrs (
              map (
                crossSystem: lib.nameValuePair "bin-${crossSystem}" (buildFor (pkgsCrossFor crossSystem))
              ) crossSystems
            ))

            # Static cross build
            // (lib.listToAttrs (
              map (
                crossSystem:
                lib.nameValuePair "static-${crossSystem}" (buildFor (pkgsCrossFor crossSystem).pkgsStatic)
              ) crossSystems
            ))

            # Docker cross build
            // (lib.listToAttrs (
              map (
                crossSystem: lib.nameValuePair "docker-${crossSystem}" (buildDockerFor (pkgsCrossFor crossSystem))
              ) crossSystems
            ));

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
