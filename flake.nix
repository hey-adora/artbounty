{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
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
              OPENSSL_NO_VENDOR = 1;
              useFetchCargoVendor = true;
              cargoHash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
              doCheck = false;

              nativeCheckInputs = [ final.nodejs_latest ];

              buildInputs = [ final.openssl ];
              src = final.fetchCrate {
                pname = "wasm-bindgen-cli";
                version = "0.2.114";
                hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
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
              perf
              samply
              surrealdb
              inotify-tools
              rust_toolchain
              wild
              clang
              taplo
              cargo-leptos
              wasm-pack
              wasm-bindgen-cli_0_2_114
              tailwindcss_4
              watchman
              yarn
              pkg-config
              openssl
              ripgrep
              #playwrith
              python312Packages.playwright
              playwright-driver.browsers
            ];
            RUST_BACKTRACE = 1;
            RUST_SRC_PATH = "${rust_toolchain}/lib/rustlib/src/rust/library";
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              pkgs.openssl
            ];
            PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
            PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
          };
      }
    );
}
