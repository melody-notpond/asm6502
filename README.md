# asm6502
6502 assembler made in Rust.

## Build
Just run `cargo build --release` if you have cargo installed. If you don't then you should [install it](https://doc.rust-lang.org/stable/book/ch01-01-installation.html) before downloading this.

## Usage
Usage: `./asm6502 [-o output] [-d] [files]`  

### Options
- `-o output`/`--out output`: Set the output file (default is `a.out`)
- `-d`/`--disc`: Set the format of the output file from a Commadore 64 object file to a full 64 kilobyte ram disc image

## Assembler Specific Pragmas
- `.define label value` - sets a label to have a value
- `.origin address` - sets the address of the following code
- `.byte byte` - appends a byte to the generated code
- `.bytes bytes` - appends a series of bytes to the generated code (comma separated, strings work)
- `.word` - appends a little endian word to the generated code
- `.include path` - includes the labels in the included file into the current file (path must be a string)
