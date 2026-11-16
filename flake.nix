{
  description = "aarch64 bare-metal OS for Raspberry Pi 4";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system:
          f (import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          }));
    in
    {
      devShells = forAllSystems (pkgs:
        let
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          x11Libs = with pkgs; [ libx11 libxi libxtst ];
        in
        {
          default = pkgs.mkShell {
            packages = [
              toolchain
              pkgs.cargo-binutils
              pkgs.just
              pkgs.qemu
              pkgs.dosfstools
              pkgs.mtools
              pkgs.util-linux
              pkgs.gdb
            ] ++ x11Libs;

            env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath x11Libs;

            shellHook = ''
              echo "rustc:  $(rustc --version)"
              echo "qemu:   $(qemu-system-aarch64 --version | head -1)"
              echo "run 'just' to list build recipes"
            '';
          };
        });
    };
}
