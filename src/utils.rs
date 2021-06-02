// Copyright 2021 Anicet Ebou.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed except according
// to those terms.

extern crate anyhow;

use anyhow::Result;

// encode_dna encode a dna string into 3 integers by Chaos Game Representation
pub fn encode_dna(seq: &str) -> Result<(i32, i32, usize)> {
    let a = vec![1, 1];
    let t = vec![-1, 1];
    let c = vec![-1, -1];
    let g = vec![1, -1];
    let mut aa;
    let mut bb;
    let mut x: Vec<i32> = Vec::new();
    let mut y: Vec<i32> = Vec::new();
    let base: i32 = 2;

    // Get sequence length
    let n = seq.len();

    for (index, nucleotide) in seq.chars().enumerate() {
        if index == 0 {
            if nucleotide == 'A' {
                aa = a[0];
                bb = a[1];
            } else if nucleotide == 'T' {
                aa = t[0];
                bb = t[1];
            } else if nucleotide == 'C' {
                aa = c[0];
                bb = c[1];
            } else {
                aa = g[0];
                bb = g[1];
            }
            x.push(aa);
            y.push(bb);
        } else {
            let new_index = index as u32;
            if nucleotide == 'A' {
                aa = x[index - 1] + base.pow(new_index);
                bb = y[index - 1] + base.pow(new_index);
            } else if nucleotide == 'T' {
                aa = x[index - 1] - base.pow(new_index);
                bb = y[index - 1] + base.pow(new_index);
            } else if nucleotide == 'C' {
                aa = x[index - 1] - base.pow(new_index);
                bb = y[index - 1] - base.pow(new_index);
            } else {
                aa = x[index - 1] + base.pow(new_index);
                bb = y[index - 1] - base.pow(new_index);
            }

            x.push(aa);
            y.push(bb);
        }
    }
    Ok((x[n - 1], y[n - 1], n))
}

pub fn decode_dna(x: i32, y: i32, n: usize) -> Result<String> {
    let mut a: Vec<i32> = vec![0; n];
    let mut b: Vec<i32> = vec![0; n];
    a[n - 1] = x;
    b[n - 1] = y;
    let base: i32 = 2;

    let mut seq = Vec::with_capacity(n);

    for index in (0..n).step_by(1).rev() {
        let nucleotide = get_nucleotide(a[index], b[index]).unwrap();
        seq.push(nucleotide);
        let (f, g) = get_cgr_vertex(a[index], b[index]).unwrap();
        if index != 0 {
            a[index - 1] = a[index] - base.pow(index as u32) * f;
            b[index - 1] = b[index] - base.pow(index as u32) * g;
        }
    }
    seq.reverse();
    let merged: String = seq.iter().collect();

    Ok(merged)

}

fn get_nucleotide(x: i32, y: i32) -> Result<char> {
    if x > 0 && y > 0 {
        Ok('A')
    } else if x > 0 && y < 0 {
        Ok('G')
    } else if x < 0 && y > 0 {
        Ok('T')
    } else if x < 0 && y < 0 {
        Ok('C')
    } else {
        Ok('N')
    }
}

fn get_cgr_vertex(x: i32, y: i32) -> Result<(i32, i32)> {

    if x > 0 && y > 0 {
        Ok((1, 1))
    } else if x > 0 && y < 0 {
        Ok((1, -1))
    } else if x < 0 && y > 0 {
        Ok((-1, 1))
    } else if x < 0 && y < 0 {
        Ok((-1, -1))
    } else {
        Ok((0, 0))
    }
}

// Tests -------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // encode_dna test
    #[test]
    fn test_encode_dna() {
        let seq = "ATCGATCGATCGATCGATCG";
        assert_eq!(encode_dna(seq).unwrap(), (209715, -629145, 20));
    }

    // decode_dna test
    #[test]
    fn test_decode_dna() {
        assert_eq!(decode_dna(209715, -629145, 20).unwrap(), "ATCGATCGATCGATCGATCG");
    }
}
