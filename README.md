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

## Cargo Features & Environment Flags
There are some compile-time features that enable shell-toy to perform certain things such as using an internal copy of fortunes embedded in the executable. This can be customized based on what you want. More details are below:
| Feature                                                                                          | Description                                                                                                                                             |
|--------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------|
| `inline-fortune`                                                                                 | Enables inlining fortunes in the compiled `sh-toy                                                                                                       |
| ` executable. See [Compiling with Inline Things](#compiling-with-inline-things) for more details |
| `inline-off-fortune`                                                                             | Also inlines Offensive fortunes. These are still blocked behind the `-o` flag                                                                           |
| `inline-cowsay`                                                                                  | Enables inlining cowsay files in the compiles `sh-toy` executable. . See [Compiling with Inline Things](#compiling-with-inline-things) for more details |

There are also some environment variables that the build script does _existence checks_ on which overrides build-script behavior. _Unless specified, the build script will only check if the variable exists and not the value. Remove the variable instead of setting it to 0 in these cases to disable the variable's behavior._

| Env Variable                         | Description                                                                                                                                                                                                                                                                               |
|--------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `USE_DEFAULT_RESOURCES`              | Will ignore `COW_PATH`, `FORTUNE_FILE`, and `FORTUNE_PATH` variables if they exist and instead use resources extracted from the archives defined in `BuildConfig.toml`. This is not required if the aforementioned variables are undefined.                                               |
| `FORCE_DOWNLOAD`                     | Will redownload resource archives even if they exist. This has no effect if "default resources" are not being used.                                                                                                                                                                       |
| `COW_PATH`                           | **_Value is used by build script._** Indicates where to find the cow files to inline in the executable if the `inline-cowsay` feature is enabled.                                                                                                                                         |
| `FORTUNE_FILE` &nbsp; `FORTUNE_PATH` | **_Value is used by build script._** Indicates where to find the cow files to inline in the executable if the `inline-fortune` feature is enabled. &nbsp; Use the `FILE` variant if all fortunes are in a single file and there is no need to differentiate fortunes as offensive or not. |

### Compiling with Inline Things
 The build script will look for the following IN THIS ORDER when it comes to using the fortunes:

1. If the `FORTUNE_FILE` environment variable is specified, the build script will only use the contents of that file.
2. If the `FORTUNE_PATH` environment variable is specified, it will embed the fortunes in that directory and subdirectory as a single file.
   1. Offensive fortunes that are in a subdirectory `off` are processed separately and only embedded if the `inline-off` feature is enabled. So if you have `FORTUNE_PATH=~/.config/fortunes`, place offensive fortunes in `~/.config/fortunes/off`
3. The build script will pull use the fortunes in the archive and internal archive path specified in `BuildConfig.toml`.


 The build script will look for the following IN THIS ORDER when it comes to using the cowsay files:


1. If the `COW_PATH` environment variable is specified, it will use the cow files specified by the environment variables.
2. The build script will pull use the fortunes in the archive and internal archive path specified in `BuildConfig.toml`.

Cow files are stored in a "map" which will support the explicit choice of choosing an embedded cow.


**NOTE: using an inline feature will remove command-line/environment variable options to look at an override path for the specific type of thing (i.e. using `inline-cowsay` will remove the ability to use a `COW_PATH` environment variable). This is an explicit choice to simplify the binary**

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