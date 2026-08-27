{
  description = "A safe JSON CLI for Apple Mail automation on macOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-25.11-darwin";
  };

  outputs =
    { nixpkgs, ... }:
    let
      system = "aarch64-darwin";
      pkgs = import nixpkgs { inherit system; };
      package = pkgs.rustPlatform.buildRustPackage {
        pname = "apple-mail-cli";
        version = "0.1.0";
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
        packages = with pkgs; [
          cargo
          cargo-deny
          cargo-machete
          clippy
          commitlint
          deadnix
          lefthook
          ls-lint
          nixfmt-rfc-style
          rustc
          rustfmt
          statix
        ];
      };
    };
}
