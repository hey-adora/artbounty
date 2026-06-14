{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    playwright.url = "github:pietdevries94/playwright-web-flake";
    playwright.inputs.nixpkgs.follows = "nixpkgs";
    # playwright.inputs.utils.follows = "utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      rust-overlay,
      playwright,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
          (final: prev: {
            inherit (playwright.packages.${system}) playwright-test playwright-driver;
          })
          (final: prev: {
            wasm-bindgen-cli_0_2_122 = final.rustPlatform.buildRustPackage {
              pname = "wasm-bindgen-cli";
              version = "0.2.122";
              OPENSSL_NO_VENDOR = 1;
              useFetchCargoVendor = true;
              cargoHash = "sha256-Inup6vvJSG5ghNyeDPyZbfZo4d0LsMG2OJfStoaeDBs=";
              doCheck = false;

              nativeCheckInputs = [ final.nodejs_latest ];

              buildInputs = [ final.openssl ];
              src = final.fetchCrate {
                pname = "wasm-bindgen-cli";
                version = "0.2.122";
                hash = "sha256-vO4RSxi/sMWxmsEs3GuljdMfIRSu75A+Q+c5wgYToRU=";
              };

            };
          })

        ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        rust_toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShell =
          with pkgs;
          mkShell {
            packages = [
              # perf
              # samply
              # surrealdb
              # inotify-tools
              rust_toolchain
              wild
              clang
              taplo
              vtsls
              emmet-language-server
              tailwindcss-language-server
              prettier
              eslint
              # cargo-leptos
              wasm-pack
              wasm-bindgen-cli_0_2_122
              tailwindcss_4
              nodejs_25
              pnpm
              # watchman
              # yarn
              pkg-config
              openssl
              ripgrep
              # playwright
              # python312Packages.playwright
              # playwright-driver.browsers
              playwright-test
            ];
            RUST_BACKTRACE = 1;
            RUST_SRC_PATH = "${rust_toolchain}/lib/rustlib/src/rust/library";
            # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            #   pkgs.openssl
            # ];
            PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = 1;
            PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
            # PLAYWRIGHT_BROWSERS_PATH = 0;
            # PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
            # PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
            # PLAYWRIGHT_NODEJS_PATH = "${pkgs.nodejs_25}/bin/node";
            # PLAYWRIGHT_LAUNCH_OPTIONS_EXECUTABLE_PATH = "${pkgs.playwright-driver.browsers}/chromium-1208";
          };
      }
    );
}
