#![allow(dead_code)]

/// Returns the greatest common divisor of two numbers.
fn gcd(a: usize, b: usize) -> usize {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Returns the number of integers <= n that are coprime with n.
fn phi(n: usize) -> usize {
    (1..=n).filter(|&i| gcd(n, i) == 1).count()
}

/// Adapted from [Joe Sawada's C implementation] and optimized for a binary alphabet.
///
/// [Joe Sawada's C implementation]: http://www.cis.uoguelph.ca/~sawada/prog/ranking_necklaces.c
pub struct NecklaceRanker<const N: usize, T> {
    divs: Vec<usize>,
    phis: Vec<T>,
}

macro_rules! impl_rank {
($($T:ty),+) => {$(
impl<const N: usize> Default for NecklaceRanker<N, $T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> NecklaceRanker<N, $T> {
    /// Creates a new `NecklaceRanker`.
    pub fn new() -> Self {
        let mut divs = Vec::new();
        let mut phis = Vec::new();
        for d in 1..=N {
            if N % d == 0 {
                divs.push(d);
                phis.push(phi(N / d) as $T);
            }
        }
        Self { divs, phis }
    }

    /// Returns the value of the `i`-th bit of `w`.
    #[inline]
    fn get(w: $T, i: usize) -> $T {
        (w >> (N - i - 1)) & 1
    }

    fn lyn_necklace(w: $T, n: usize) -> (usize, bool) {
        let mut p = 1;
        for i in 1..n {
            let u = Self::get(w, i);
            let v = Self::get(w, i - p);
            if u < v {
                return (p, false);
            } else if u > v {
                p = i + 1;
            }
        }
        (p, true)
    }

    /// Computes the largest necklace <= w[..n].
    fn largest_necklace(w: $T, n: usize) -> $T {
        let mut res = w;
        let (mut p, mut done) = Self::lyn_necklace(res, n);
        while !done {
            // res[p - 1] = 0;
            res &= !(1 << (N - p));
            // res[p..n].fill(1);
            res |= (1 << (N - p)) - 1;
            (p, done) = Self::lyn_necklace(res, n);
        }
        res
    }

    /// Returns the number of strings whose necklace is <= w.
    fn t(w: $T, n: usize) -> $T {
        let mut suf = [[0; N]; N];
        let mut b = [[0; N]; N];
        let mut s;
        let mut tot: $T;

        // Sets neck[..n] to the largest necklace less than or equal to w[..n]
        let neck = Self::largest_necklace(w, n);
        // let neck = w; // ???

        // Compute b[t][j] = number of strings of length t with prefix neck[..j] but no suffix less than neck[..n]
        // OPT: B[t][j] -> B[t-1][j], unroll 1st loop
        for t in 0..n {
            b[t][t] = 1 - Self::get(neck, t) as $T; // 0 + X * 1
            for j in (0..t).rev() {
                b[t][j] = b[t][j + 1] + (1 - Self::get(neck, j) as $T) * b[t - j - 1][0];
            }
        }

        // Compute suf[i][j] = longest suffix of neck[i..j] that is a prefix of neck[..n]
        for i in 1..n {
            s = i;
            for j in i..n {
                if Self::get(neck, j) > Self::get(neck, j - s) {
                    s = j + 1;
                }
                suf[i][j] = j + 1 - s;
            }
        }

        // Compute t
        tot = Self::lyn_necklace(neck, n).0 as $T;
        // OPT: B[t][j] -> B[t-1][j], unroll 1st loop
        // for j in 0..n {
        //     tot += Self::get(neck, j) << (n - 1 - j);
        // }
        tot += neck >> (N - n);
        for t in 2..=n {
            for j in 0..n {
                if j + t <= n {
                    tot += Self::get(neck, j) * b[t - 2][0] << (n - t - j);
                } else {
                    if j < n - t + 2 {
                        s = 0;
                    } else {
                        s = suf[n - t + 1][j - 1];
                    }
                    if Self::get(neck, j) > Self::get(neck, s) {
                        tot += b[n - j + s - 1][s + 1];
                    }
                }
            }
            // for j in 0..=(n - t) {
            //     tot += Self::get(neck, j) * b[t - 2][0] << (n - t - j);
            // }
            // // j = n - t + 1
            // s = 0;
            // if Self::get(neck, n - t + 1) > Self::get(neck, s) {
            //     tot += b[t - 1 + s - 1][s + 1];
            // }
            // for j in (n - t + 2)..n {
            //     s = suf[n - t + 1][j - 1];
            //     if Self::get(neck, j) > Self::get(neck, s) {
            //         tot += b[n - j + s - 1][s + 1];
            //     }
            // }
        }
        tot
    }

    /// Returns the rank of a necklace.
    pub fn rank(&self, w: $T) -> $T {
        let mut r = 0;
        for (&d, phi) in self.divs.iter().zip(self.phis.iter()) {
            r += phi * Self::t(w, d);
        }
        r / (N as $T) - 1
    }
}
)*}}

impl_rank!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi() {
        for n in 0..10 {
            println!("{}", phi(n));
        }
    }

    // #[test]
    // fn test_rank_lmers() {
    //     use crate::utils::all_lmers;
    //     type T = u32;
    //     const K: usize = 9;
    //     const N: usize = 2 * K - 1;
    //     let ranker = NecklaceRanker::<N, T>::new();
    //     all_lmers::<K>().iter().enumerate().for_each(|(i, &x)| {
    //         println!("{}\t{:b}", ranker.rank(x), x);
    //         assert_eq!(ranker.rank(x), i as T);
    //     });
    // }
}
