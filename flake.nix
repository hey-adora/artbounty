{
  inputs = {
    # naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils,
    # naersk,
    rust-overlay, }:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          # (final: prev: {
          #   dioxus-cli-0_7 = final.rustPlatform.buildRustPackage {
          #     pname = "dioxus-cli";
          #     version = "0.7.0-alpha.1";
          #     nativeCheckInputs = [ final.rustfmt ];
          #     OPENSSL_NO_VENDOR = 1;
          #     useFetchCargoVendor = true;
          #     cargoHash = "sha256-r42Z6paBVC2YTlUr4590dSA5RJJEjt5gfKWUl91N/ac=";
          #     buildFeatures = [
          #       "no-downloads"
          #       "optimizations"
          #     ];
          #
          #     nativeBuildInputs = [
          #       final.pkg-config
          #       final.cacert
          #     ];
          #
          #     buildInputs = [ final.openssl ];
          #     src = final.fetchCrate {
          #       pname = "dioxus-cli";
          #       version = "0.7.0-alpha.1";
          #       hash = "sha256-3b82XlxffgbtYbEYultQMzJRRwY/I36E1wgzrKoS8BU=";
          #     };
          #
          #     checkFlags = [
          #       # requires network access
          #       "--skip=serve::proxy::test"
          #       "--skip=wasm_bindgen::test"
          #     ];
          #   };
          # })

        ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        # naersk-lib = pkgs.callPackage naersk { };
        rust_toolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        # defaultPackage = naersk-lib.buildPackage ./.;
        devShell = mkShell {
          packages = with pkgs; [
            surrealdb
            inotify-tools
            rust_toolchain
            mold
            clang
            # llvmPackages.bintools
            # dioxus-cli-0_7
            cargo-leptos
            wasm-pack
            wasm-bindgen-cli_0_2_100
            tailwindcss_4
            watchman
            yarn
            pkg-config
            openssl
            # openssl.dev
            # bun
            # nodejs_24
            # pre-commit
            # alsa-lib
            # libudev-zero
            # # lld
            # sqlite
            # fontconfig
            # freetype
            # #wayland
            # wayland
            # libxkbcommon
            # #x11
            # # xorg.libX11
            # # xorg.libX11
            # # xorg.libXcursor
            # # xorg.libXi
            # # xorg.libXrandr
            # #vulkan
            # vulkan-tools
            # vulkan-headers
            # vulkan-loader
            # vulkan-validation-layers
            # #opengl
            # libGL
            #playwrith
            python312Packages.playwright
            playwright-driver.browsers
          ];
          RUST_BACKTRACE = 1;
          RUST_SRC_PATH = "${rust_toolchain}/lib/rustlib/src/rust/library";
          # RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold -Z share-generics=y";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.openssl
            # pkgs.openssl.dev
            # pkgs.wayland
            # pkgs.libxkbcommon
            # pkgs.vulkan-loader
            # pkgs.alsa-lib
            # pkgs.udev
          ];
          PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
          PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
        };
      });
}
