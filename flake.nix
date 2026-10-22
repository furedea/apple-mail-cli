{
  description = "A safe JSON CLI for Apple Mail automation on macOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-26.05-darwin";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { nixpkgs, nixpkgs-unstable, ... }:
    let
      system = "aarch64-darwin";
      pkgs = import nixpkgs { inherit system; };
      # Must match cargo-dist-version in Cargo.toml and CARGO_DIST_VERSION in release_artifacts.yml.
      cargoDist = pkgs.cargo-dist.overrideAttrs (
        finalAttrs: _: {
          version = "0.30.2";
          src = pkgs.fetchFromGitHub {
            owner = "axodotdev";
            repo = "cargo-dist";
            tag = "v${finalAttrs.version}";
            hash = "sha256-1stUmm7rtNB2z2srOzDvQ9QaGsS0CySBOHt118vmJoM=";
          };
          cargoHash = "sha256-Il5PVJHoNdifqUcXxKR+j+Lgga0kIsl7IJp9oGanZ+c=";
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit (finalAttrs) src;
            hash = finalAttrs.cargoHash;
          };
        }
      );
      package = pkgs.rustPlatform.buildRustPackage {
        pname = "apple-mail-cli";
        inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) version;
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        meta = {
          description = "A safe JSON CLI for Apple Mail automation on macOS";
          homepage = "https://github.com/furedea/apple-mail-cli";
          license = pkgs.lib.licenses.mit;
          mainProgram = "apple-mail";
          platforms = pkgs.lib.platforms.darwin;
        };
      };
    in
    {
      packages.${system}.default = package;

      apps.${system}.default = {
        type = "app";
        program = "${package}/bin/apple-mail";
        meta.description = "Read and organize accounts configured in Apple Mail";
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = [
          cargoDist
          # lefthook before 2.1.6 runs hooks under a pty, which agent sandboxes deny.
          nixpkgs-unstable.legacyPackages.${system}.lefthook
        ]
        ++ (with pkgs; [
          cargo
          cargo-deny
          cargo-machete
          clippy
          commitlint
          deadnix
          ls-lint
          nixfmt
          nodejs_22
          pnpm
          rustc
          rustfmt
          statix
        ]);
      };
    };
}
