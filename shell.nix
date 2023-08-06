{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  packages = with pkgs; [ 
    cargo 
    rustc 
    /* libclang */
    rust-analyzer
  ];

  buildInputs = with pkgs; [ rustfmt clippy ] ++ lib.optional stdenv.isDarwin libiconv;
}
