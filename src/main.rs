// Copyright 2021 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.s

mod utils;

fn main() {
    println!("{:?}", utils::encode_dna("ATCGATCGATCGATCGATCG").unwrap());
    println!("{:?}", utils::decode_dna(209715, -629145, 20).unwrap());
}
