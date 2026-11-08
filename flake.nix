{
  description = "discloud-rs development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            }
          )
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "discloud-dev";
          packages = [
            # Same channel and components as rust-toolchain.toml, plus editor tooling.
            ((pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
            })
            pkgs.cargo-deny
            # Only needed for the optional `native-tls` feature (`--all-features`).
            pkgs.pkg-config
            pkgs.openssl
          ];
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
