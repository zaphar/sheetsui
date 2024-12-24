{
  inputs = {
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk.url = "github:nix-community/naersk";
    flake-utils = {url = "github:numtide/flake-utils"; };
  };

  outputs = {self, nixpkgs, flake-utils, rust-overlay, naersk}:
  flake-utils.lib.eachDefaultSystem (system: let
    overlays = [
      rust-overlay.overlays.default
    ];
    pkgs = import nixpkgs { inherit system overlays; };
    my-rust-bin = pkgs.rust-bin.stable.latest.default;
    naersk-lib = pkgs.callPackage naersk {
      rustc = my-rust-bin;
      cargo = my-rust-bin;
    };
    project = naersk-lib.buildPackage {
    	src = ./.;
      root = ./.;
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs =
        (
          if pkgs.stdenv.isDarwin
          then with pkgs.darwin.apple_sdk.frameworks; [Security SystemConfiguration]
          else [pkgs.openssl]
        )
        ++ [my-rust-bin];
    };
  in
  {
    packages.default = project;
    rust-bin = pkgs.rust-bin;
    
    devShells.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [ gnumake my-rust-bin rust-analyzer cargo-tarpaulin ];
    };
  });
}
