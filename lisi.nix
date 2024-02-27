# import niv sources and the pinned nixpkgs
{ sources ? import ./nix/sources.nix, pkgs ? import sources.nixpkgs { }}:

let
  # import rust compiler
  rust = import ./nix/rust.nix { inherit sources; };

  # configure naersk to use our pinned rust compiler
  naersk = pkgs.callPackage sources.naersk {
    rustc = rust;
    # We would use our rust package here. However it
    # doesent supports --out-dir as a cargo option right now.
    # So we fall back to the buildin cargo
    # see https://github.com/nmattia/naersk/issues/100
    #cargo = rust;
    cargo = pkgs.cargo;
  };

  # tell nix-build to ignore the `target` directory
  src = builtins.filterSource
    (path: type: type != "directory" || builtins.baseNameOf path != "target")
    ./.;
in naersk.buildPackage {
  name = "lisi-rust-workspace";

  inherit src;
  remapPathPrefix =
    true; # remove nix store references for a smaller output package
}
