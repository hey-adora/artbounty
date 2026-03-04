{
  inputs = {
    # naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      # naersk,
      rust-overlay,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
          (final: prev: {
            wasm-bindgen-cli_0_2_114 = final.rustPlatform.buildRustPackage {
              pname = "wasm-bindgen-cli";
              version = "0.2.114";
              # nativeCheckInputs = [ final.rustfmt ];
              OPENSSL_NO_VENDOR = 1;
              useFetchCargoVendor = true;
              cargoHash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
              doCheck = false;
              # buildFeatures = [
              #   "no-downloads"
              #   "optimizations"
              # ];

              nativeCheckInputs = [ final.nodejs_latest ];

              # nativeBuildInputs = [
              #   final.pkg-config
              #   final.cacert
              # ];

              buildInputs = [ final.openssl ];
              src = final.fetchCrate {
                pname = "wasm-bindgen-cli";
                version = "0.2.114";
                hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
              };

              # checkFlags = [
              #   # requires network access
              #   "--skip=serve::proxy::test"
              #   "--skip=wasm_bindgen::test"
              # ];
            };
          })

        ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        # naersk-lib = pkgs.callPackage naersk { };
        rust_toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        # defaultPackage = naersk-lib.buildPackage ./.;
        devShell =
          with pkgs;
          mkShell {
            packages = [
              # aws-lc
              perf
              samply
              # cargo-flamegraph
              surrealdb
              inotify-tools
              rust_toolchain
              wild
              # mold
              clang
              taplo
              # llvmPackages.bintools
              # dioxus-cli-0_7
              cargo-leptos
              wasm-pack
              wasm-bindgen-cli_0_2_114
              # wasm-bindgen-cli_0_2_100
              tailwindcss_4
              watchman
              yarn
              pkg-config
              openssl
              ripgrep
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
              # pkgs.aws-lc
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
      }
    );
}
