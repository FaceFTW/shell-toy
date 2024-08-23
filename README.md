# shell-toy
A `cowsay | fortune` implementation in Rust.

***THIS IS STILL A WIP***
![Demo](./demo.gif)
(Pardon the glitches, those are from the asciinema recording)

## Installation

Currently, pre-built binaries are not provided since the project is in a state I don't consider to be "finished".

For now the best way is to use Cargo:
```
cargo install shell-toy
```
This will install shell-toy to your local path as `sh-toy`. You can then put it in whatever terminal startup script you want.

## Cargo Features
There are some compile-time features that enable shell-toy to perform certain things such as using an internal copy of fortunes embedded in the executable. This can be customized based on what you want. More details are below:
| Feature       | Description                                                                                                                                           |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `inline`      | Enables inlining fortunes in the compiled `sh-toy` executable. See [Compiling with inline fortunes](#compiling-with-inline-fortunes) for more details |
| `inline-off`  | Also inlines Offensive fortunes. These are still blocked behind the `-o` flag                                                                         |
| `fortune-git` | Only uses the fortunes from the [`fortune-mod`](https://github.com/shlomif/fortune-mod) repository. Expects the submodule in the repo to be cloned    |

### Compiling with Inline Fortunes
 The build script will look for the following IN THIS ORDER when it comes to using the fortunes:

1. If the `FORTUNE_FILE` environment variable is specified, the build script will only use the contents of that file.
2. If the `FORTUNE_PATH` OR `FORTUNEPATH` environment variables is specified, it will embed the fortunes in that directory and subdirectory as a single file.
   1. Offensive fortunes that are in a subdirectory `off` are processed separately and only embedded if the `inline-off` feature is enabled. So if you have `FORTUNE_PATH=~/.config/fortunes`, place offensive fortunes in `~/.config/fortunes/off`
3. If on a Linux platform, it will look in the default fortune location at `/usr/share/games/fortunes`. Offensive fortunes from here will still not be embedded unless you enable the `inline-off` feature.


If you enable the `fortune-git` feature and clone the `fortune-mod` submodule, None of the above steps will be followed. Like in Step 3 from above, offensive fortunes from here will still not be embedded unless you enable the `inline-off` feature.

If you choose to compile on any non-Linux plaforms, the build script will require one of the above environment variables to be specified UNLESS you enable the `fortune-git` feature and clone the `fortune-mod` submodule.

---
## Usage

Help is available by running with the `--help` flag
```
 $ sh-toy.exe --help
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

## Legal Stuff
`shell-toy` is licensed under the MIT License.

While `fortune-mod` is a Git submodule of this repository, no source code or binary linkage occurs with the `fortune-mod` executables; only the fortune files are used in certain compilation scenarios. I claim no ownership or intellectual property of content in that repository.

This project uses modified portions of code from the following projects (this is documented in the source code where it occurs)
- [charasay](https://github.com/latipun7/charasay/blob/main/src/bubbles.rs) - MIT License