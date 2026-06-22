{
  # USAGE: `nix develop`
  # supported architectures: ["x86_64-linux" "aarch64-darwin" "x86_64-darwin"]

  description = "metassr - a simple dev shell with rust and metacall";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, ... }:
    flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-darwin" "x86_64-darwin"] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        metacallConfig = {
          "x86_64-linux" = {
            libPaths = ["/gnu/"];
            dynlibVar = "LD_LIBRARY_PATH";
          };
          "aarch64-darwin" = {
            libPaths = ["/opt/homebrew/lib/"];
            dynlibVar = "DYLD_LIBRARY_PATH";
          };
          "x86_64-darwin" = {
            libPaths = ["/usr/local/lib/"];
            dynlibVar = "DYLD_LIBRARY_PATH";
          };
        };

        cfg = metacallConfig.${system};

        formatLibPaths = paths: builtins.concatStringsSep ":" paths;

        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-nYxm7Okhb4WOD0C/qCJ3uzm+VwgQTG4SSpO8IXewVXU=";
        };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "metassr-dev";

          buildInputs = with pkgs; [
            # Rust toolchain from fenix
            rustToolchain
            # Build tools
            pkg-config cmake gcc
            # System libs
            openssl libffi llvmPackages.libclang
            # Runtimes
            nodejs_22
            # dev tools
            git curl
            less
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS =
            "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include";

          env = {
            # rust-src is included in the fenix toolchain
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };

          shellHook = ''
            export ${cfg.dynlibVar}=${formatLibPaths cfg.libPaths}:$${cfg.dynlibVar}
            export LIBRARY_PATH=${formatLibPaths cfg.libPaths}:$LIBRARY_PATH
            export RUSTFLAGS="${builtins.concatStringsSep " " (map (path: "-L ${path}") cfg.libPaths)}"

            if [ -n "$BASH_VERSION" ]; then
              export PS1="\[\033[1;32m\][metassr-dev]\[\033[0m\]:\[\033[1;34m\]\w\[\033[0m\] $ "
            elif [ -n "$ZSH_VERSION" ]; then
              export PROMPT="%F{green}[metassr-dev]%f:%F{blue}%~%f $ "
            fi

            echo "Welcome to dev shell (${system})"
            echo "Node.js: $(node --version)"
            echo "Rust: $(rustc --version)"
          '';
        };

        checks = {
          devShell-evaluates = pkgs.runCommand "devShell-check" {} ''
            echo "devShell for ${system} evaluates correctly"
            touch $out
          '';

          rust-toolchain-ok = pkgs.runCommand "rust-toolchain-check" {} ''
            ${rustToolchain}/bin/rustc --version > $out
          '';
        };
      });
}