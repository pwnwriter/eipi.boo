{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems =
        function: nixpkgs.lib.genAttrs systems (system: function (import nixpkgs { inherit system; }));
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {

          packages = with pkgs; [
            # flyctl
          ];

          shellHook = ''
            echo "------eipi.boo--------"
          '';

          # env = {
          #   # Linux
          #   LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          #     pkgs.stdenv.cc.cc.lib
          #   ];
          #   # MacOS
          #   DYLD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          #     pkgs.stdenv.cc.cc.lib
          #   ];
          # };
        };
      });
    };
}
