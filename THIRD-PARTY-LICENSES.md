# Third-Party Licenses and Acknowledgements

This project uses third-party software and resources. Below are the acknowledgements and license information.

## Syntax Definitions

The syntax highlighting feature uses syntax definitions (`assets/syntaxes.bin`) from the [bat](https://github.com/sharkdp/bat) project.

- **Source**: https://github.com/sharkdp/bat
- **License**: MIT License / Apache License 2.0 (dual licensed)
- **Copyright**: Copyright (c) 2018-2024 bat-developers
- **Usage**: The `syntaxes.bin` file contains compiled syntax definitions originally from Sublime Text packages

### bat License (MIT)

```
MIT License

Copyright (c) 2018-2024 bat-developers

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Sublime Text Syntax Definitions

The syntax definitions in `syntaxes.bin` are derived from Sublime Text packages, which are primarily licensed under the MIT License.

- **Source**: https://github.com/sublimehq/Packages
- **License**: MIT License (for most packages)
- **Copyright**: Various authors (see individual package repositories)

## Rust Dependencies

This project also uses various Rust crates. For a complete list of dependencies and their licenses, run:

```bash
cargo tree --format "{p} ({l})"
```

### Key Dependencies

- **syntect** (MIT) - Syntax highlighting engine
- **encoding_rs** (Apache-2.0 OR MIT) - Character encoding support
- **anyhow** (MIT OR Apache-2.0) - Error handling
- **pico-args** (MIT OR Apache-2.0) - Command-line argument parsing
- **once_cell** (MIT OR Apache-2.0) - Lazy static initialization
- **ansi_colours** (LGPL-3.0-or-later) - ANSI color conversion

## Notes

- The `syntaxes.bin` file is a binary compilation of `.sublime-syntax` files
- The original syntax definitions are maintained by their respective communities
- cate uses these resources in compliance with their permissive open source licenses
- We are grateful to the bat project and the Sublime Text community for maintaining these excellent syntax definitions

## Compliance

This project complies with all license requirements:
- MIT License allows redistribution with attribution
- Attribution is provided in this file and in the README
- Original copyright notices are preserved
- No warranty claims are made

If you have concerns about licensing, please open an issue at the project repository.
