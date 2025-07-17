# 🧬 chaoscoder

**DNA sequence encoding, decoding, and analysis using (Integer) Chaos Game Representation**

[![CI](https://github.com/Ebedthan/chaoscoder/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/chaoscoder/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/chaoscoder/graph/badge.svg?token=K7VN5TH6EZ)](https://codecov.io/gh/Ebedthan/chaoscoder)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat)](https://github.com/Ebedthan/chaoscoder/blob/master/LICENSE)


## 🌟 Overview

`chaoscoder` is a high-performance Rust tool for transforming DNA sequences into visual or numerical formats using **Chaos Game Representation (CGR)** and its improved version, **Integer Chaos Game Representation (iCGR)**. It is the **first user-oriented implementation** of iCGR, supporting encoding, decoding, visualization, and comparison workflows.

### ✨ Features

- ✅ **iCGR Encoding/Decoding** of DNA sequences using a robust, lossless integer-based method
- ✅ **Efficient block-based encoding** for long sequences (100 bp windows)
- ✅ **CGR image generation** from DNA sequences
- ✅ **Similarity analysis** using the [DSSIM algorithm](https://github.com/kornelski/dssim) for comparing CGR images


## 🚀 Installation

You’ll need a [Rust](https://rust-lang.org/tools/install) toolchain (`stable` channel).

```bash
git clone https://github.com/Ebedthan/chaoscoder.git
cd chaoscoder
cargo build --release
```


## Installation

```bash
git clone https://github.com/Ebedthan/chaoscoder.git
cd chaoscoder
cargo build --release
```

## 🛠️ Usage

Here are the main commands available

```bash
# Encode a FASTA sequence to iCGR format
chaoscoder encode seq.fa

# Decode an iCGR file back to the original sequence
chaoscoder decode seq.icgr

# Generate a CGR image from a DNA sequence
chaoscoder draw seq.fa

# Compare CGR images in a folder using DSSIM
chaoscoder compare images_dir
```

For full details, do `chaoscoder -h`.

## 📦 Requirements

Rust ≥ 1.82.0 (minimum supported version)
Optional: fontconfig and pkg-config (for CGR rendering, may require system dependencies)


### Minimum Rust version policy
This crate's minimum supported `rustc` version is `1.82.0`.


### 🐛 Bugs & Feedback

Found a bug? Have a feature request?
Open an issue on the GitHub [Issue Tracker](https://github.com/Ebedthan/chaoscoder/issues).


### 📄 License
This project is licensed under the MIT License.
See [LICENSE-MIT](LICENSE-MIT) (or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT)) for details.
