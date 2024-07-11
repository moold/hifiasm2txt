# hifiasm2txt
A small tool to convert hifiasm bin files (`ec.bin`, `ovlp.source.bin` and `ovlp.reverse.bin`) to text format.

## Installation

#### Dependencies

`hifiasm2txt` is written in rust, try below commands (no root required) or see [here](https://www.rust-lang.org/tools/install) to install `Rust` first.
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Download and install

```
git clone git@github.com:moold/hifiasm2txt.git
cd hifiasm2txt && cargo build --release
```
*If you want to speed up downloading and compiling in China, please visit [here](https://mirrors.tuna.tsinghua.edu.cn/help/crates.io-index/) to set up a mirror (Tsinghua Open Source Mirror).*

## Usage
```
./target/release/hifiasm2txt path_to_hifiasm_workdir/hifiasm.asm --out_ec_bin --out_so_bin --out_re_bin
```

## Parameters
```
A small tool to convert hifiasm bin files to text (gzip) format.

Usage: hifiasm2txt [OPTIONS] <PATH/PREFIX> [PREFIX]

Arguments:
  <PATH/PREFIX>  output file path and prefix of `hifiasm`.
  [PREFIX]       prefix of output files. [default: hifiasm2txt]

Options:
  -e, --out_ec_bin  convert PATH/PREFIX.ec.bin file.
  -r, --out_re_bin  convert PATH/PREFIX.reverse.bin file.
  -s, --out_so_bin  convert PATH/PREFIX.source.bin file.
  -h, --help        Print help
  -V, --version     Print version
```