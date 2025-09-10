{
  # USAGE: `nix develop .#metassr`

  description = "metassr - a simple dev shell with rust and metacall";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";

    in
    {
      devShells."x86_64-linux".default = pkgs.mkShell {
        name = "metassr-dev";

        # tools for rust development
        buildInputs = with pkgs; [
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer
          pkg-config
          openssl
          git
          curl
        ];
        env = {
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
        shellHook = ''
          # export PATH="$HOME/.local/bin:$PATH"

          # change $PS1 if user uses bash
          if [ -n "$BASH_VERSION" ]; then
          export PS1="\[\033[1;32m\](metassr-dev)\[\033[0m\]:\[\033[1;34m\]\w\[\033[0m\] $ "
          # change $PROMPT if user uses zsh
          elif [ -n "$ZSH_VERSION" ]; then
            export PROMPT="%F{green}(metassr-dev)%f:%F{blue}%~%f $ "
          fi

          echo "welcome to dev shell"
        '';
      };
    };
}