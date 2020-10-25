# asm6502
6502 assembler made in Rust.

## Build
Just run `cargo build --release` if you have cargo installed. If you don't then you should [install it](https://doc.rust-lang.org/stable/book/ch01-01-installation.html) before downloading this.

## Usage
Usage: `./asm6502 [-o output] [-d] [files]`  

### Options
- `-o output`/`--out output`: Set the output file (default is `a.out`)
- `-d`/`--disc`: Stops the assembler from setting the first two bytes of the file to be the address where the program is in memory and outputs the entire 64 kilobyte RAM disc image
- `-s addr`/`--start addr`: Sets the first address to be placed in the output (inclusive; must be hexadecimal)
- `-e addr`/`--end addr`: Sets the last address to be placed in the output (inclusive; must be hexadecimal)

## Assembler Specific Pragmas
- `.define label value` - sets a label to have a value
- `.origin address` - sets the address of the following code
- `.byte byte` - appends a byte to the generated code
- `.bytes bytes` - appends a series of bytes to the generated code (comma separated, strings work)
- `.word` - appends a little endian word to the generated code
- `.include path` - includes the labels in the included file into the current file (path must be a string)
