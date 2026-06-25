# wlmstr

A stateful CLI tool for managing wallpapers using awww. It cycles through images in a specified directory and changes the wallpaper each time it is run. Supported slide modes include sequential, reverse, and random order.

The application's state is persisted in XDG_DATA_HOME/wlmstr/data.json.

## TODO

- [ ]: check work video mode
- [ ]: enhance help and error printings
- [x]: support flake pkg
- [ ]: feat: custom wallapepr contol command by config file(now, only support awww and mpbpaper)

## Install

### with Home Manager

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

### with nix profile

```bash
nix profile install github:Uliboooo/wlmstr
```

## Usage

```zsh
:) wlmstr --help
Stateful wallpaper slideshow CLI for awww

Usage: wlmstr <COMMAND>

Commands:
  next    go to next slide
  status  Get status. supports to output in JSON or debug format
  set     set config data
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```zsh
~
:) wlmstr status
current dir: /home/seli/Pictures/KAF/
current WallPaper: /home/seli/Pictures/KAF/kininaru-anoko.jpg
Mode: Image
~
:) wlmstr next seq
change to /home/seli/Pictures/KAF/mid-night.jpg
~
:) wlmstr status
current dir: /home/seli/Pictures/KAF/
current WallPaper: /home/seli/Pictures/KAF/mid-night.jpg
Mode: Image
~
:) wlmstr set -d ~/Pictures/wallpapers/
change to /home/seli/Pictures/wallpapers/画像-4.jpg
~
:) wlmstr status
current dir: /home/seli/Pictures/wallpapers/
current WallPaper: /home/seli/Pictures/wallpapers/画像-4.jpg
Mode: Image
~
:) wlmstr next seq
change to /home/seli/Pictures/wallpapers/0c12ebbdb79a6cfe509231c7bba738f8.jpg
~
:) wlmstr status
current dir: /home/seli/Pictures/wallpapers/
current WallPaper: /home/seli/Pictures/wallpapers/0c12ebbdb79a6cfe509231c7bba738f8.jpg
Mode: Image
```
