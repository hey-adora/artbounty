{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
      rust-overlay,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
        ];

        pkgs = import nixpkgs { inherit system overlays; };
        naersk-lib = pkgs.callPackage naersk { };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              rust
              mold
              cargo-leptos
              tailwindcss_4
              yarn
              pre-commit
              openssl.dev
              pkg-config
              alsa-lib
              libudev-zero
              clang
              lld
              sqlite
              fontconfig
              freetype
              #wayland
              wayland
              libxkbcommon
              #x11
              # xorg.libX11
              # xorg.libX11
              # xorg.libXcursor
              # xorg.libXi
              # xorg.libXrandr
              #vulkan
              vulkan-tools
              vulkan-headers
              vulkan-loader
              vulkan-validation-layers
              #opengl
              libGL
              #playwrith
              python312Packages.playwright
              playwright-driver.browsers
            ];
            RUST_BACKTRACE = 1;
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            # RUSTFLAGS = "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.vulkan-loader
              pkgs.alsa-lib
              pkgs.udev
            ];
            PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
            PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
          };
      }
    );
}
