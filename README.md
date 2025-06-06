[![Rust](https://github.com/BrandonTang89/Y86-64-Seq/actions/workflows/rust.yml/badge.svg)](https://github.com/BrandonTang89/Y86-64-Seq/actions/workflows/rust.yml)

# Assembler for Y86-64

## Overview
Simple assembler for Y86-64 Language, a simplified version of the x86-64 assembly language used in the book "Computer Systems: A Programmer's Perspective" (CS:APP).

Instructions and encoding can be found in `docs`.

## Usage
```bash
cargo run -- <path-to-asm-file> (<output-file>)
```

### Example Usage
```bash
cargo run -- examples/hello.ys
xxd examples/hello.yso
```

Should give you an output like this:
```
00000000: 7030 0000 0000 0000 0000 0000 0000 0000  p0..............
00000010: 0100 0000 0000 0000 0200 0000 0000 0000  ................
00000020: 0300 0000 0000 0000 0400 0000 0000 0000  ................
00000030: 30f4 1000 0000 0000 0000 30f5 0400 0000  0.........0.....
00000040: 0000 0000 804e 0000 0000 0000 0090 30f8  .....N........0.
00000050: 0800 0000 0000 0000 30f9 0100 0000 0000  ........0.......
00000060: 0000 6300 6255 707f 0000 0000 0000 0050  ..c.bUp........P
00000070: a400 0000 0000 0000 0060 a060 8461 9574  .........`.`.a.t
00000080: 6f00 0000 0000 0000 90                   o........
```

## Internals
Uses Chumsky, a parser combinator library, to parse the Y86-64 assembly language. The assembler translates the parsed instructions into binary format according to the encoding rules specified in the documentation.

# Instruction Level Simulator for Y86-64
Todo.