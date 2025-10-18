{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = with pkgs; [ pkg-config openssl ];
  shellHook = ''
  	set -a
	source .env
	set +a
  '';
}
