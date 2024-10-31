{
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          toolDeps = with pkgs; [
            gcc
          ];
          runtimeDeps = with pkgs; [
          ];
          libraryDeps = with pkgs; [
          ];
          libPath = pkgs.lib.makeLibraryPath libraryDeps;
        in
        {
          devShell = pkgs.mkShell {
            packages = toolDeps ++ runtimeDeps ++ libraryDeps;
            RUST_LOG = "error";
            LD_LIBRARY_PATH = "${libPath}";
          };
        }
      );
}
