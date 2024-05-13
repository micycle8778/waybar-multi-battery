{
  description = "Custom battery status module for waybar that handles multiple batteries.";
  
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { system = system; };
    rustPlatform = pkgs.rustPlatform;
  in
  {
    packages.${system}.default = rustPlatform.buildRustPackage {
      pname = "waybar-multi-battery";
      version = "0.2.0";
      src = ./.;

      cargoLock.lockFile = ./Cargo.lock;

      meta = {
        description = "Custom battery status module for waybar that handles multiple batteries.";
        homepage = "https://github.com/micycle8778/waybar-multi-battery";
        license = nixpkgs.lib.licenses.mit;
      };
    };
    # packages.${system}.default = pkgs.stdenv.mkDerivation {
    #   name = "waybar-multi-battery";
    #   src = ./.;
    #
    #   buildInputs = with pkgs; [
    #     rustc
    #     cargo
    #   ];
    #
    #   buildPhase = ''
    #     cargo build --release
    #   '';
    #
    #   installPhase = ''
    #     mkdir -p $out/bin
    #     cp $src/target/release/waybar-multi-battery $out/bin
    #   '';
    # };

    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        rustc
        cargo
      ];
    };
  };
}
