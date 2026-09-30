//! xoshiro128** seeded exactly like the oracle's `game/core/chance.ts`, so
//! both engines draw identical outcomes from the same seed when draws happen
//! in the same order.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng {
    s: [u32; 4],
}

impl Rng {
    pub fn new(seed: u32) -> Rng {
        let mut s = [0u32; 4];
        let mut x = seed;
        for v in s.iter_mut() {
            x = x.wrapping_add(0x9e37_79b9);
            let mut z = x;
            z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
            z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
            *v = z ^ (z >> 16);
        }
        Rng { s }
    }

    pub fn next_u32(&mut self) -> u32 {
        let s = &mut self.s;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 9;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(11);
        result
    }

    /// Uniform integer in [0, n) by rejection sampling (no draw when n <= 1).
    pub fn below(&mut self, n: u32) -> u32 {
        if n <= 1 {
            return 0;
        }
        let limit = (0x1_0000_0000u64 / n as u64) * n as u64;
        let mut v = self.next_u32() as u64;
        while v >= limit {
            v = self.next_u32() as u64;
        }
        (v % n as u64) as u32
    }

    pub fn coin(&mut self) -> bool {
        self.below(2) == 0
    }

    /// Fisher–Yates permutation of `0..n`, as `RngSource.shuffle`.
    pub fn shuffle(&mut self, n: usize, out: &mut [u8]) {
        for (i, o) in out.iter_mut().enumerate().take(n) {
            *o = i as u8;
        }
        let mut i = n;
        while i > 1 {
            i -= 1;
            let j = self.below(i as u32 + 1) as usize;
            out.swap(i, j);
        }
    }

    pub fn index(&mut self, n: usize) -> usize {
        self.below(n as u32) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_oracle_stream() {
        // Values produced by the TypeScript Rng(1): first shuffle of 60 from
        // the tier-1 trace (seed 1) starts 20,33,8,5,2,...
        let mut r = Rng::new(1);
        r.coin(); // setup: who begins
        let mut out = [0u8; 60];
        r.shuffle(60, &mut out);
        assert_eq!(&out[..5], &[20, 33, 8, 5, 2]);
    }
}
