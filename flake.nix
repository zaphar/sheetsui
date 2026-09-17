{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {url = "github:numtide/flake-utils"; };
  };

  outputs = {self, nixpkgs, flake-utils, rust-overlay, crane}:
  flake-utils.lib.eachDefaultSystem (system: let
    overlays = [
      rust-overlay.overlays.default
    ];
    pkgs = import nixpkgs { inherit system overlays; };
    inherit (pkgs) lib;
    my-rust-bin = pkgs.rust-bin.stable."1.92.0".default;
    craneLib = (crane.mkLib pkgs).overrideToolchain my-rust-bin;

    # src/ui/help/mod.rs reads the help text out of docs/ with
    # include_str!. The usual cargo source filter keeps only Rust and
    # Cargo files, so it drops docs/ and the build fails on files that
    # are present on disk. Add docs/ back.
    src = lib.fileset.toSource {
      root = ./.;
      fileset = lib.fileset.unions [
        (craneLib.fileset.commonCargoSources ./.)
        ./docs
      ];
    };

    # No platform-specific buildInputs are necessary.
    #
    # Darwin: the Apple SDK comes from the stdenv, which gives every
    # derivation AppKit, Security and SystemConfiguration. Add
    # `pkgs.apple-sdk_15` (or later) to buildInputs only to ask for a
    # newer SDK than the stdenv default.
    #
    # Linux: nothing links against OpenSSL. arboard reaches Wayland and
    # X11 through dlopen and pure-Rust protocol crates, so it needs no
    # library at build time.
    commonArgs = {
      inherit src;
      strictDeps = true;
      nativeBuildInputs = [pkgs.pkg-config];
    };

    # Built once and reused by the package and every check.
    cargoArtifacts = craneLib.buildDepsOnly commonArgs;

    sheetui = craneLib.buildPackage (commonArgs // {
      inherit cargoArtifacts;
      doCheck = false;
    });
  in
  {
    packages.default = sheetui;
    rust-bin = pkgs.rust-bin;

    checks = {
      inherit sheetui;

      clippy = craneLib.cargoClippy (commonArgs // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      });

      test = craneLib.cargoTest (commonArgs // {
        inherit cargoArtifacts;
        # The clipboard tests reach for the system pasteboard, which the
        # sandbox does not have. They carry #[serial], and every one of
        # them has copy or paste in its name.
        cargoTestExtraArgs = "-- --skip copy --skip paste";
      });
    };

    devShells.default = craneLib.devShell {
      packages = with pkgs; [
        pkg-config
        gnumake
        rust-analyzer
        cargo-tarpaulin
      ];
    };
  });
}
