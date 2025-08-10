{
  description = "Simple tool that finds and switches the current SSH_AUTH_SOCK variable. Useful for tmux users.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system: 
  let
    pkgs = import nixpkgs {
      inherit system;
    };
  in
  {
    packages.default = pkgs.rustPlatform.buildRustPackage {
      name = "refresh-auth-sock";
      version = "0.1.0";
      cargoHash = "sha256-U8QyHJO6U+QK63LJ+hd2sphm8KDBrOVSzqnF+zmN+q4=";
      nativeBuildInputs = with pkgs; [ cargo ];
      src = ./.;
    };
  });
}
