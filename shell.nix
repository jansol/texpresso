with import <nixos> {};
{ pkgs ? import <nixpkgs> {} }:
let
in
stdenv.mkDerivation {
   name = "texpresso";
}
