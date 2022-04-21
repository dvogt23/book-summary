{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    fenix.inputs.rust-analyzer-src.follows = "rustAnalyzer";
    flakeCompat.url = github:edolstra/flake-compat;
    flakeCompat.flake = false;
    flakeUtils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rustAnalyzer.url = "github:rust-analyzer/rust-analyzer";
    rustAnalyzer.flake = false;
    treefmt.url = "github:numtide/treefmt";
    treefmt.inputs.flake-utils.follows = "flakeUtils";
    treefmt.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs =
    inputs:
    inputs.flakeUtils.lib.eachSystem
      [ "x86_64-darwin" "x86_64-linux" ]
      (
        system:
        let
          nixpkgs = import inputs.nixpkgs { inherit system; };
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          treefmt = inputs.treefmt.defaultPackage.${ system };
          fenix = inputs.fenix.packages.${ system };
          fenixPlatform = nixpkgs.makeRustPlatform { inherit (fenix.latest) cargo rustc; };
        in
          {
            checks = { defaultPackage = inputs.self.defaultPackage.${ system }; };
            defaultApp = {
              type = "app";
              program = "${ inputs.self.defaultPackage.${ system } }/bin/book-summary";
            };
            defaultPackage =
              fenixPlatform.buildRustPackage
                {
                  pname = cargoToml.package.name;
                  version =
                    let
                      commit = inputs.self.shortRev or "dirty";
                      date = inputs.self.lastModifiedDate or inputs.self.lastModified or "19700101";
                    in
                      "${ builtins.substring 0 8 date }_${ commit }";
                  src = inputs.self.sourceInfo;
                  cargoLock.lockFile = ./Cargo.lock;
                  meta = {
                    description = cargoToml.package.description;
                    homepage = "https://github.com/dvogt23/book-summary";
                    license = nixpkgs.lib.licenses.mit;
                  };
                };
            devShell =
              nixpkgs.mkShell
                {
                  name = "Book-Summary";
                  packages = [
                    fenix.rust-analyzer
                    fenix.latest.cargo
                    fenix.latest.clippy
                    fenix.latest.rust-src
                    fenix.latest.rustc
                    fenix.latest.rustfmt
                    nixpkgs.cargo-tarpaulin
                    nixpkgs.jq
                    nixpkgs.nodejs
                    nixpkgs.nodePackages.prettier
                    nixpkgs.nodePackages.prettier-plugin-toml
                    nixpkgs.shfmt
                    nixpkgs.glibc
                    treefmt
                  ];
                };
          }
      );
}

