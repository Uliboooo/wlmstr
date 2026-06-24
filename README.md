# wlmstr

A stateful CLI tool for managing wallpapers using awww. It cycles through images in a specified directory and changes the wallpaper each time it is run. Supported slide modes include sequential, reverse, and random order.

The application's state is persisted in XDG_DATA_HOME/wlmstr/data.json.

## TODO

- [ ]: check work video mode
- [ ]: enhance help and error printings
- [ ]: support flake pkg
- [ ]: feat: custom wallapepr contol command by config file(now, only support awww and mpbpaper)

## Install with Home Manager

Add `wlmstr` to your `flake.nix` inputs:

```nix
{
  description = "Example configuration for wlmstr";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    wlmstr = {
      url = "github:Uliboooo/wlmstr";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

Then add the package to your Home Manager configuration:

```nix
# home/user.nix
{
  home.packages = [
    inputs.wlmstr.packages.${pkgs.system}.default
  ];
}
```

## Run

```bash
nix run github:Uliboooo/wlmstr
```

## Install

```bash
nix profile install github:Uliboooo/wlmstr
```
