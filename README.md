# chaoscoder

*DNA Sequence encoding, decoding and analysis using (Integer) Chaos Game Representation*

[![Continuous Integration](https://github.com/Ebedthan/chaoscoder/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/chaoscoder/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/chaoscoder/branch/main/graph/badge.svg?token=K7VN5TH6EZ)](https://codecov.io/gh/Ebedthan/chaoscoder)
<a href="https://github.com/Ebedthan/chaoscoder/blob/master/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat">
</a>
<br/>

## 🗺️ Overview
`chaoscoder` implement [integer chaos game representation (iCGR) algorithm](https://www.liebertpub.com/doi/abs/10.1089/cmb.2018.0173) for DNA sequence encoding and decoding. `chaoscoder` is the first complete implementation of the algorithm in a bioinformatic tool aiming at users. It also add to the original algorithm a output file format which is a `zst` compressed JSON file containing the 3 integers of 100bp subsequences of the supplied sequence. This allow fast encoding and decoding.

`chaoscoder` also implements [chaos game representation (CGR) of DNA sequence](https://academic.oup.com/nar/article-abstract/18/8/2163/2383530) in a fast tool that draw the representation of a sequence and can compare the CGR image using the [DSSIM algorithm](https://github.com/kornelski/dssim/).

## Installation

```bash
git clone https://github.com/Ebedthan/chaoscoder.git
cd chaoscoder
cargo build --release
```

## User guide

```bash
# Encoding DNA sequence into integer chaos game representation
chaoscoder encode seq.fa

# Decoding integer chaos game representation into DNA sequence
chaoscoder decode seq.icgr

# Draw chaos game representation of DNA sequence
chaoscoder draw seq.fa

# Compare multiple chaos game representation image using DSSIM
chaoscoder compare images_dir
```

For full details, do `chaoscoder -h`.

### Requirements
- [Rust](https://rust-lang.org) in stable channel


### Minimum Rust version policy
This crate's minimum supported `rustc` version is `1.82.0`.


### Bugs
Submit problems or requests to the [Issue Tracker](https://github.com/Ebedthan/chaoscoder/issues).


### License
Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT).
