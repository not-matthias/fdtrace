{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: let
  unstable = import inputs.nixpkgs-unstable {system = pkgs.stdenv.system;};
in {
  # https://devenv.sh/packages/
  packages = with pkgs; [
    bpftrace
    cargo-insta
  ];

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "nightly";
    targets = ["x86_64-unknown-linux-gnu"];
    components = ["rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src"];
  };

  env.RUST_LOG = "info";
}
