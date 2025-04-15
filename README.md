# fastchaos
<a href="https://github.com/Ebedthan/fastchaos/actions?query=workflow%3A%22Continuous+Integration%22">
    <img src="https://img.shields.io/github/actions/workflow/status/Ebedthan/fastchaos/ci.yml?style=flat&logo=GitHub%20Actions&branch=main">
</a>
<a href="https://codecov.io/gh/Ebedthan/fastchaos">
    <img src="https://codecov.io/gh/Ebedthan/fastchaos/branch/main/graph/badge.svg?token=K7VN5TH6EZ"/>
</a>
<a href="https://github.com/Ebedthan/fastchaos/blob/master/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat">
</a>
<br/>

## 🗺️ Overview
`fastchaos` implement [integer chaos game representation (iCGR) algorithm](https://www.liebertpub.com/doi/abs/10.1089/cmb.2018.0173) for DNA sequence encoding and decoding. `fastchaos` is the first complete implementation of the algorithm in a bioinformatic tool aiming at users. It also add to the original algorithm a output file format which is a `zst` compressed JSON file containing the 3 integers of 100bp subsequences of the supplied sequence. This allow fast encoding and decoding.

`fastchaos` also implements [chaos game representation (CGR) of DNA sequence](https://academic.oup.com/nar/article-abstract/18/8/2163/2383530) in a fast tool that draw the representation of a sequence and can compare the CGR image using the [DSSIM algorithm](https://github.com/kornelski/dssim/).

## Installation

```bash
git clone https://github.com/Ebedthan/fastchaos.git
cd fastchaos
cargo build --release
```

## User guide

```bash
# Encoding DNA sequence into integer chaos game representation
fastchaos encode seq.fa

# Decoding integer chaos game representation into DNA sequence
fastchaos decode seq.icgr

# Draw chaos game representation of DNA sequence
fastchaos draw seq.fa

# Compare multiple chaos game representation image using DSSIM
fastchaos compare images_dir
```

For full details, do `fastchaos -h`.

### Requirements
- [Rust](https://rust-lang.org) in stable channel


### Minimum Rust version policy
This crate's minimum supported `rustc` version is `1.82.0`.


### Bugs
Submit problems or requests to the [Issue Tracker](https://github.com/Ebedthan/fastchaos/issues).


### License
Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT).
