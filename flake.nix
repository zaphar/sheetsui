{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {url = "github:numtide/flake-utils"; };
  };

  outputs = {self, nixpkgs, flake-utils, rust-overlay, naersk}:
  flake-utils.lib.eachDefaultSystem (system: let
    overlays = [
      rust-overlay.overlays.default
    ];
    pkgs = import nixpkgs { inherit system overlays; };
    my-rust-bin = pkgs.rust-bin.stable."1.92.0".default;
    naersk-lib = pkgs.callPackage naersk {
      rustc = my-rust-bin;
      cargo = my-rust-bin;
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
    project = naersk-lib.buildPackage {
      src = ./.;
      root = ./.;
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = [my-rust-bin];
    };
  in
  {
    packages.default = project;
    rust-bin = pkgs.rust-bin;

    devShells.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        pkg-config
        gnumake
        my-rust-bin
        rust-analyzer
        cargo-tarpaulin
      ];
    };
  });
}
