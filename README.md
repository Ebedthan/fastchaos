# fastchaos
<a href="https://github.com/Ebedthan/fastchaos/actions?query=workflow%3A%22Continuous+Integration%22">
    <img src="https://img.shields.io/github/workflow/status/Ebedthan/fastchaos/Continuous%20Integration?style=flat&logo=GitHub%20Actions">
</a>
<a href="https://github.com/Ebedthan/fastchaos/actions?query=workflow%3A%22Continuous+Deployment%22">
    <img src="https://img.shields.io/github/workflow/status/Ebedthan/fastchaos/Continuous%20Deployment?style=flat&logo=GitHub%20Actions&label=deploy">
</a>
<a href="https://crates.io/crates/fastchaos">
    <img src="https://img.shields.io/crates/v/fastchaos.svg?style=flat">
</a>
<a href="https://codecov.io/gh/Ebedthan/fastchaos">
    <img src="https://codecov.io/gh/Ebedthan/fastchaos/branch/main/graph/badge.svg?token=K7VN5TH6EZ"/>
</a>
<a href="https://github.com/Ebedthan/fastchaos/blob/master/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat">
</a>
<br/>

## 1 - About

## 2 - Installation

### Using cargo

```bash
cargo install fastchaos
```

### Downloading the precompiled releases

See [release](https://github.com/Ebedthan/fastchaos) page to download your prefered binary.


### Compile it yourself

```bash
git clone https://github.com/Ebedthan/fastchaos.git

cd fastchaos

cargo build --release
```

## 3 - Usage

### Encoding DNA sequence into integer chaos game representation

```bash
fastchaos encode seq.fa
```

### Decoding integer chaos game representation into DNA sequence

```bash
fastchaos decode seq.icgr
```

### Draw chaos game representation of DNA sequence

```bash
fastchaos draw seq.fa
```

### Compare multiple chaos game representation image using DSSIM

```bash
fastchaos compare images_dir
```

For full details, do `fastchaos -h`.

### Requirements
- [Rust](https://rust-lang.org) in stable channel


### Minimum Rust version policy
This crate's minimum supported `rustc` version is `1.57.0`.


### Note
`fastchaos` use colored output in help, nevertheless `fastchaos` honors [NO_COLORS](https://no-color.org/) environment variable.


### Bugs
Submit problems or requests to the [Issue Tracker](https://github.com/Ebedthan/fastchaos/issues).


### License
Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT).
