<picture>
  <source media="(prefers-color-scheme: light)" srcset="https://github.com/cloudwego/volo/raw/main/.github/assets/volo-light.png?sanitize=true" />
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/cloudwego/volo/raw/main/.github/assets/volo-dark.png?sanitize=true" />
  <img alt="Volo" src="https://github.com/cloudwego/volo/raw/main/.github/assets/volo-light.png?sanitize=true" />
</picture>

`volo-cli` is the command line tool for Volo.

`volo-cli` provides the ability to generate default project layout and manage the idls used.

## Install

Simply run:

```bash
$ cargo install volo-cli
```

## Usage

```
USAGE:
    volo [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    Turn on the verbose mode.
    -V, --version    Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    idl     manage your idl
    init    init your project
    repo    manage your repo
    migrate auto migrate from the previous config to the latest one
```

For more detailed examples, you can check the documentation (TODO).
