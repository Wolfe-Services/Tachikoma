# Spec 537: Nix Integration

## Overview
Nix package manager integration for reproducible development environments, hermetic builds, and declarative system configuration in Tachikoma deployments.

## Requirements

### Nix Flake Definition
```nix
{
  description = "Tachikoma autonomous development agent";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.buildGoModule {
          pname = "tachikoma";
          version = "0.1.0";
          src = ./.;
          vendorHash = "sha256-...";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            go
            gopls
            golangci-lint
            protobuf
            grpcurl
          ];
        };
      }
    );
}
```

### Development Shell
```go
type NixDevShell struct {
    Name         string   `json:"name"`
    Packages     []string `json:"packages"`
    ShellHook    string   `json:"shellHook,omitempty"`
    EnvVars      map[string]string `json:"envVars,omitempty"`
    PureDeps     bool     `json:"pureDeps"`
}
```

### Nix Builder Interface
```go
type NixBuilder interface {
    // Build derivation
    Build(ctx context.Context, flakeRef string, attr string) (*BuildResult, error)

    // Enter development shell
    DevShell(ctx context.Context, flakeRef string) (*ShellSession, error)

    // Evaluate expression
    Eval(ctx context.Context, expr string) (interface{}, error)

    // Run in nix shell
    Run(ctx context.Context, packages []string, cmd []string) (*ExecResult, error)
}
```

### Build Result
```go
type BuildResult struct {
    StorePath    string    `json:"storePath"`
    Outputs      map[string]string `json:"outputs"`
    BuildTime    time.Duration `json:"buildTime"`
    Size         int64     `json:"size"`
    References   []string  `json:"references"`
}
```

### NixOS Module
```nix
{ config, lib, pkgs, ... }:

{
  options.services.tachikoma = {
    enable = lib.mkEnableOption "Tachikoma agent";

    config = lib.mkOption {
      type = lib.types.attrs;
      default = {};
      description = "Tachikoma configuration";
    };
  };

  config = lib.mkIf config.services.tachikoma.enable {
    systemd.services.tachikoma = {
      description = "Tachikoma Agent";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.tachikoma}/bin/tachikoma agent";
        Restart = "always";
      };
    };
  };
}
```

### Binary Cache
```go
type BinaryCache interface {
    // Check if path exists in cache
    Exists(ctx context.Context, storePath string) (bool, error)

    // Upload to cache
    Upload(ctx context.Context, storePath string) error

    // Configure cache
    Configure(ctx context.Context, config CacheConfig) error
}

type CacheConfig struct {
    URL        string `json:"url"`
    PublicKey  string `json:"publicKey"`
    SecretKey  string `json:"secretKey,omitempty"`
    Priority   int    `json:"priority"`
}
```

### Integration Features
- Automatic flake.nix generation
- devenv.sh compatibility
- direnv integration
- CI/CD Nix builds
- Remote builders

### Reproducibility
- Pinned nixpkgs revision
- Flake lock file
- Hermetic builds
- Content-addressed derivations

## Dependencies
- None (build system)

## Verification
- [ ] Flake builds successfully
- [ ] Dev shell functional
- [ ] NixOS module works
- [ ] Binary cache uploads
- [ ] Reproducible builds
