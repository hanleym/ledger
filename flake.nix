{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix.url = "github:nix-community/fenix/monthly";
    crane.url = "github:ipetkov/crane";
    nixpkgs.follows = "fenix/nixpkgs";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = inputs @ { self, flake-parts, advisory-db, ... }: flake-parts.lib.mkFlake { inherit inputs; } (top @ { config, withSystem, moduleWithSystem, ... }: {
    systems = [
      #"aarch64-linux"
      "x86_64-linux"
      #"aarch64-darwin"
      #"x86_64-darwin"
    ];
    perSystem = perSystem @ { config, system, lib, pkgs, ... }: let
      fenixToolchain = inputs.fenix.packages.${system}.latest.withComponents [
        "cargo"
        "clippy"
        "rustc"
        "rust-src"
        "rustfmt"
        "llvm-tools-preview"
      ];
      rustToolchain = pkgs.symlinkJoin {
        name = "rust-toolchain";
        paths = with pkgs; [
          fenixToolchain
          cargo-llvm-cov
          cargo-tarpaulin
        ];
      };
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      version = self.rev or self.dirtyRev;
      src = ./.;

      commonArgs = {
        inherit version src;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      ledger = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
        meta.mainProgram = "ledger";
        doCheck = false;
      });
    in {
      checks = {
        inherit ledger;

        clippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

        docs = craneLib.cargoDoc (commonArgs // {
          inherit cargoArtifacts;
          env.RUSTDOCFLAGS = "--deny warnings";
        });

        fmt = craneLib.cargoFmt {
          inherit src;
        };

        toml-fmt = craneLib.taploFmt {
          src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
        };

        audit = craneLib.cargoAudit {
          inherit src advisory-db;
        };

        nextest = craneLib.cargoNextest (commonArgs // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
          cargoNextestPartitionsExtraArgs = "--no-tests=pass";
        });
      };

      devShells.default = craneLib.devShell rec {
        checks = self.checks.${system};
        shellHook = ''
          ln -sfn "${rustToolchain}" "$PWD/.rust"
          export RUST_SRC_PATH="$PWD/.rust/lib/rustlib/src/rust/library"
        '';

        buildInputs = with pkgs; [];

        LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
      };

      packages = {
        inherit ledger;
        default = ledger;
      };
    };
  });
}
