{ pkgs ? import <nixpkgs> {} }:

let
  metacallConfig = {
    # override this by your needs
    defaultLibPaths = [
      "/usr/local/lib"
      "~/.local/lib"
    ];
    
    # Version specifications for key dependencies
    versions = {
      nodejs = "22";  # Default Node.js version
    };
  };

  # Helper function to format library paths
  formatLibPaths = paths: builtins.concatStringsSep ":" paths;

  # Core development dependencies
  coreDependencies = with pkgs; [
    pkg-config
    cmake
    gcc
  ];

  # Language runtimes required by MetaCall
  runtimeDependencies = with pkgs; [
    nodejs
    nodejs_22  # Consider making this configurable based on metacallConfig
  ];

  # Build and compilation tools
  buildTools = with pkgs; [
    libclang
    clang
    cargo
    rustc
    rust-analyzer
  ];

  # System libraries and their development versions
  systemLibraries = with pkgs; [
    openssl
    openssl.dev
    libffi
    libffi.dev
    llvmPackages.libclang
  ];

in pkgs.mkShell {
  # Combine all dependencies
  buildInputs = coreDependencies 
    ++ runtimeDependencies 
    ++ buildTools 
    ++ systemLibraries;

  # Shell environment setup
  shellHook = ''
    # Set up environment variables for library paths
    export LD_LIBRARY_PATH=${formatLibPaths metacallConfig.defaultLibPaths}:$LD_LIBRARY_PATH
    export LIBRARY_PATH=${formatLibPaths metacallConfig.defaultLibPaths}:$LIBRARY_PATH
    
    # Set up Rust-specific configurations
    export RUSTFLAGS="${builtins.concatStringsSep " " (map (path: "-L ${path}") metacallConfig.defaultLibPaths)}"
    
    # Add any local bin directories to PATH
    export PATH=$PATH:$HOME/.local/bin
    
    # Print helpful information when entering the shell
    echo "MetaCall development environment activated!"
    echo "Node.js version: $(node --version)"
    echo "Rust version: $(rustc --version)"
  '';

  # Include any additional environment variables that might be needed
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include";
}