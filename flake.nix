{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
  }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-linux"
    ] (
      system: let
        inherit (nixpkgs) lib;

        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        workspaceCargoToml = lib.importTOML ./Cargo.toml;
        testMembers =
          lib.filter (m: !(lib.hasSuffix "/afl" m) && m != "testsuite")
          workspaceCargoToml.workspace.members;

        targets = ["thumbv7em-none-eabi" "thumbv6m-none-eabi"];

        craneLib = (crane.mkLib pkgs).overrideToolchain (pkgs.rust-bin.stable.latest.default.override {
          inherit targets;
        });

        # https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu.html
        craneLibNightly = (crane.mkLib pkgs).overrideToolchain (pkgs.rust-bin.nightly.latest.default.override {
          inherit targets;
        });

        src = craneLib.cleanCargoSource ./.;

        cargoToml = {
          dhcp = lib.importTOML ./dhcp/Cargo.toml;
          dns = lib.importTOML ./dns/Cargo.toml;
          fuzz = lib.importTOML ./fuzz/Cargo.toml;
          hl = lib.importTOML ./hl/Cargo.toml;
          ll = lib.importTOML ./ll/Cargo.toml;
          mqtt = lib.importTOML ./mqtt/Cargo.toml;
          regsim = lib.importTOML ./regsim/Cargo.toml;
          sntp = lib.importTOML ./sntp/Cargo.toml;
          testsuite = lib.importTOML ./testsuite/Cargo.toml;
          tls = lib.importTOML ./tls/Cargo.toml;
          tls-afl = lib.importTOML ./tls/afl/Cargo.toml;
        };

        features = with lib;
          mapAttrs (crate: toml: (sort lessThan (attrNames (toml.features or {})))) cargoToml;

        allFeatures = with lib;
          sort lessThan (unique (flatten (attrValues features)));

        noStdFeatures = [
          "defmt"
          "p256-cm4"
        ];
        nightlyFeatures = [
          "async"
          "eha0"
        ];

        filterNoStdFeatures = lib.filter (m: !(lib.elem m noStdFeatures));
        mkFeatures = lib.concatStringsSep ",";

        allStdCompatFeatures = mkFeatures (filterNoStdFeatures allFeatures);
        allStdStableCompatFeatures = mkFeatures (
          lib.filter (m: !(lib.elem m (noStdFeatures ++ nightlyFeatures))) allFeatures
        );

        cargoArtifactsNightly = craneLibNightly.buildDepsOnly {
          inherit src;
          cargoExtraArgs = "--features ${allStdCompatFeatures}";
        };

        cargoArtifacts = craneLibNightly.buildDepsOnly {
          inherit src;
          cargoExtraArgs = "--features ${allStdStableCompatFeatures}";
        };
      in {
        packages = {
          testsuite = craneLib.buildPackage {
            inherit src;
            inherit cargoArtifacts;
            cargoExtraArgs = "-p testsuite";
          };

          # TODO: check (v6, v7 x std, nightly)
          ll = craneLib.buildPackage {
            inherit src;
            inherit cargoArtifacts;
            cargoExtraArgs = "-p w5500-ll --target thumbv6m-none-eabi";
          };
        };

        checks = let
          nixSrc = nixpkgs.lib.sources.sourceFilesBySuffices ./. [".nix"];

          tests = lib.listToAttrs (lib.forEach testMembers (p: {
            name = "test-${p}";
            value = craneLibNightly.cargoTest {
              pname = "w5500-${p}";
              inherit src;
              cargoArtifacts = cargoArtifactsNightly;
              cargoExtraArgs = let
                featuresNoDefmt = mkFeatures (filterNoStdFeatures (lib.getAttr p features));
                featureArgs =
                  if featuresNoDefmt != ""
                  then "--features ${featuresNoDefmt}"
                  else "";
              in "-p w5500-${p} ${featureArgs}";
            };
          }));

          generatedChecks = tests;
        in
          lib.recursiveUpdate generatedChecks
          {
            clippy = craneLibNightly.cargoClippy {
              inherit src;
              cargoArtifacts = cargoArtifactsNightly;
              cargoClippyExtraArgs = "--all-features --all-targets -- --deny warnings";
            };

            rustfmt = craneLibNightly.cargoFmt {inherit src;};

            docs = craneLibNightly.cargoDoc {
              inherit src;
              cargoArtifacts = cargoArtifactsNightly;

              RUSTDOCFLAGS = "-D warnings --cfg docsrs";

              cargoExtraArgs = "--all-features";
            };

            testsuite-build = self.packages.${system}.testsuite;

            alejandra = pkgs.runCommand "alejandra" {} ''
              ${pkgs.alejandra}/bin/alejandra --check ${nixSrc}
              touch $out
            '';

            statix = pkgs.runCommand "statix" {} ''
              ${pkgs.statix}/bin/statix check ${nixSrc}
              touch $out
            '';
          };
      }
    );
}
