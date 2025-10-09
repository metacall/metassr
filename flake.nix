{
  # USAGE: `nix develop`
  # this flake only runs for architecture "x86_64-linux"

  description = "metassr - a simple dev shell with rust and metacall";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        system = "x86_64-linux"; 
        pkgs = nixpkgs.legacyPackages.${system};

        metacallConfig = {
          defaultLibPaths = [
            "/gnu/"
          ];
        };

        formatLibPaths = paths: builtins.concatStringsSep ":" paths;
      in
      {
        devShells.default = pkgs.mkShell {
          name = "metassr-dev";

          buildInputs = with pkgs; [
            # Core Rust tools
            rustc cargo rustfmt clippy rust-analyzer
            # Build tools (REVIEW THIS)
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
            RUST_SRC_PATH =
              "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };

          shellHook = ''
            # Adjust library paths for MetaCall
            export LD_LIBRARY_PATH=${formatLibPaths metacallConfig.defaultLibPaths}:$LD_LIBRARY_PATH
            export LIBRARY_PATH=${formatLibPaths metacallConfig.defaultLibPaths}:$LIBRARY_PATH
            export RUSTFLAGS="${builtins.concatStringsSep " " (map (path: "-L ${path}") metacallConfig.defaultLibPaths)}"

            # Prompt (bash vs zsh)
            if [ -n "$BASH_VERSION" ]; then
              export PS1="\[\033[1;32m\][metassr-dev]\[\033[0m\]:\[\033[1;34m\]\w\[\033[0m\] $ "
            elif [ -n "$ZSH_VERSION" ]; then
              export PROMPT="%F{green}[metassr-dev]%f:%F{blue}%~%f $ "
            fi

            echo "Welcome to dev shell"
            echo "Node.js: $(node --version)"
            echo "Rust: $(rustc --version)"
          '';
        };
      });
}
