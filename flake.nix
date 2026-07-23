{
  description = "rust-sstp向けの再現可能なRust開発環境";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        lib = pkgs.lib;
        fuzzRustToolchain = pkgs.rust-bin.nightly.latest.default;
        cargoFuzzNightly = pkgs.writeShellScriptBin "cargo-fuzz" ''
          export PATH="${fuzzRustToolchain}/bin:$PATH"
          exec ${pkgs.cargo-fuzz}/bin/cargo-fuzz "$@"
        '';
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rust-sstp";
          version = "0.1.0";
          src = lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [
            "--workspace"
            "--all-targets"
          ];
          cargoTestFlags = [
            "--workspace"
          ];
          meta.license = lib.licenses.asl20;
        };

        checks.package = self.packages.${system}.default;

        devShells.default = pkgs.mkShell {
          packages =
            (with pkgs; [
              cargo
              cargo-audit
              cargo-deny
              cargoFuzzNightly
              cargo-nextest
              clippy
              direnv
              jq
              python3
              ripgrep
              rust-analyzer
              rustc
              rustfmt
            ])
            ++ lib.optionals pkgs.stdenv.isLinux (
              with pkgs;
              [
                iproute2
                iperf3
                pkg-config
                tcpdump
              ]
            )
            ++ lib.optionals pkgs.stdenv.isDarwin (
              with pkgs;
              [
                libiconv
                pkg-config
              ]
            );

          shellHook = ''
            export RUST_BACKTRACE=1
          '';
        };
      }
    );
}
