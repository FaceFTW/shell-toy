# shell-toy
A `cowsay | fortune` implementation in Rust.

***THIS IS STILL A WIP***
[![asciicast](https://asciinema.org/a/v32df6bplfuee34x6DfE0BDLl.svg)](https://asciinema.org/a/v32df6bplfuee34x6DfE0BDLl)

## Installation

Currently, pre-built binaries are not provided since the project is in a state I don't consider to be "finished".

For now the best way is to use Cargo:
```
cargo install shell-toy
```
This will install shell-toy to your local path as `sh-toy`. You can then put it in whatever terminal startup script you want.

Help is also available by running with the `--help` flag
```
 pwsh - awest❯ sh-toy.exe --help
Usage: sh-toy.exe [<message>] [-c <cow-file>] [--cow-path <cow-path>] [-o <include-offensive>] [-b <bubble>]

various program options

Positional Arguments:
  message

Options:
  -c, --cow-file    path to a direct cowfile
  --cow-path        path to a folder containing multiple cows we should search.
  -o, --include-offensive
                    whether to include offensive fortunes
  -b, --bubble      the type of bubble to create. Options are "think", "round",
                    and "cowsay"
  --help            display usage information
```

### NOTE: Windows/Non-Linux Support

If you are on a Linux Platform which has the `cowsay` and `fortune` packages available and installed on the system, shell-toy will automatically pull from the default installation directories. Otherwise, it requires some variables or command line arguments.


- `COWPATH` or `COW_PATH`: Folder containing `cowsay` cows
- `FORTUNE_FILE`: Singular `%` delimited file for fortunes, prioritized over the `PATH` counterpart
- `FORTUNEPATH` or `FORTUNE_PATH`: Folder containing fortunes. Offensive fortunes should be placed in a child drirectory called `off` similar to how `fortune` does it.