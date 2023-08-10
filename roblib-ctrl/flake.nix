{
  description = "A breadboard controller for a dank engine";

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let pkgs = nixpkgs.legacyPackages.${system}; in {
      devShells.default = pkgs.mkShell {
        buildInputs = [ pkgs.pkg-config pkgs.udev.dev ];
      };
    });
}
