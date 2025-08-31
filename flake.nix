{
description = "Implementation of the BWINF 2024 task 'Weniger Krumme Touren' in Rust";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit overlays system;
        };
        nativeBuildInputs = with pkgs; [
          pkg-config
          gcc
          rust-bin.stable.latest.default
        ];
        buildInputs = with pkgs; [
        ];
        projectName = "weniger_krumm_rust";
        libraryPath = pkgs.lib.makeLibraryPath buildInputs;
      in
      {
        devShells.default = pkgs.mkShell {
          LD_LIBRARY_PATH = "${libraryPath}:$LD_LIBRARY_PATH";
          inherit buildInputs nativeBuildInputs;
        };
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = projectName;
          version = "0.1.0";
          cargoLock.lockFile = ./Cargo.lock;
          src = ./.;
          inherit buildInputs nativeBuildInputs;

          postFixup = ''
            patchelf --set-rpath "${libraryPath}" $out/bin/"${projectName}"
          '';
        };
        apps.default = {
          type = "app";
          program = "${self.packages.x86_64-linux.default}/bin/${projectName}";
        };
      }
    );
}
