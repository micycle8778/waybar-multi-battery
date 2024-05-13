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
      version = "0.2.1";
      src = ./.;

      cargoLock.lockFile = ./Cargo.lock;

      nativeBuildInputs = with pkgs; [ pkg-config installShellFiles makeWrapper ];

      postFixup = ''
        wrapProgram $out/bin/waybar-multi-battery \
          --set PATH ${pkgs.lib.makeBinPath (with pkgs; [
            upower
        ])}
      '';

      meta = {
        description = "Custom battery status module for waybar that handles multiple batteries.";
        homepage = "https://github.com/micycle8778/waybar-multi-battery";
        license = nixpkgs.lib.licenses.mit;
      };
    };

    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        rustc
        cargo
      ];
    };
  };
}
