{ profile ? "default" }:
let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { overlays = [ (import sources.rust-overlay) ]; };
in
pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.rust-bin.stable.latest.${profile}

    # keep this line if you use bash
    pkgs.bashInteractive
  ];
}

