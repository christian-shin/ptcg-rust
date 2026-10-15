//! xoshiro128** seeded from the game's seed: the same seed draws the same outcomes when draws happen in the same
//! order, which is how a recorded trace replays.

/// One recorded chance outcome.
#[derive(Clone, Debug)]
pub enum Draw {
    Coin(bool),
    Shuffle(Vec<u8>),
    /// `(n, value)`: a uniform pick in `0..n`.
    Index(usize, usize),
}

/// The replay tape: the recorded outcomes still to be fed, and the draws it had no outcome for.
struct Tape {
    draws: Vec<Draw>,
    misses: usize,
}

thread_local! {
    /// Recorded outcomes a replay feeds back (`diff`'s
    /// observable mode). While set, every real draw takes the first outcome
    /// of its kind and size instead of the seeded stream, so a change in when
    /// a coin, shuffle or random pick happens does not shift the others.
    static TAPE: std::cell::RefCell<Option<Tape>> = const { std::cell::RefCell::new(None) };
}

thread_local! {
    /// Outcomes drawn by recording generators ([`Rng::record`]) on this thread, in draw order:
    /// a Rust-recorded trace's chance events (`selfplay`).
    static RECORDED: std::cell::RefCell<Vec<Draw>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Take the outcomes recorded on this thread since the last call.
pub fn take_recorded() -> Vec<Draw> {
    RECORDED.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

fn record(rec: bool, d: impl FnOnce() -> Draw) {
    if rec {
        RECORDED.with(|c| c.borrow_mut().push(d()));
    }
}

/// Set (or clear, with `None`) the replay tape for this thread; its miss count starts at 0.
pub fn set_tape(t: Option<Vec<Draw>>) {
    TAPE.with(|c| *c.borrow_mut() = t.map(|draws| Tape { draws, misses: 0 }));
}

/// The draws made on this thread since the tape was set that it had no outcome of their kind and size for (served by a
/// forced coin or the seeded stream instead): a replay that matches its recording has none. 0 without a tape. A legality
/// trial's fixed generator never reads the tape, so its draws never count.
pub fn tape_misses() -> usize {
    TAPE.with(|c| c.borrow().as_ref().map_or(0, |t| t.misses))
}

/// The outcomes on this thread's tape that no draw has taken yet (0 without a tape).
pub fn tape_left() -> usize {
    TAPE.with(|c| c.borrow().as_ref().map_or(0, |t| t.draws.len()))
}

/// The first outcome on the tape that `want` accepts, taken off the tape; `None` without a tape, or when the tape has none
/// (a miss, counted). Nothing is written without a tape.
fn take_from_tape(want: impl Fn(&Draw) -> bool) -> Option<Draw> {
    TAPE.with(|c| {
        let mut t = c.borrow_mut();
        let tape = t.as_mut()?;
        match tape.draws.iter().position(want) {
            Some(k) => Some(tape.draws.remove(k)),
            None => {
                tape.misses += 1;
                None
            }
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng {
    s: [u32; 4],
    /// Coin results forced by a scenario (bit i = flip i, 1 = heads), used
    /// before the real stream.
    forced: u32,
    nforced: u8,
    /// Record every real outcome ([`take_recorded`]). Only the game's own generator records, so a
    /// policy's picks and legality trials (the fixed generator) stay out of the trace.
    rec: bool,
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
        Rng { s, forced: 0, nforced: 0, rec: false }
    }

    /// This generator with outcome recording on.
    pub fn record(mut self) -> Rng {
        self.rec = true;
        self
    }

    /// Whether this generator records its outcomes.
    pub fn recording(&self) -> bool {
        self.rec
    }

    pub fn set_record(&mut self, on: bool) {
        self.rec = on;
    }

    /// Force the next real coin flips (up to 32).
    pub fn force_coins(&mut self, coins: &[bool]) {
        self.forced = 0;
        self.nforced = coins.len().min(32) as u8;
        for (i, &c) in coins.iter().take(32).enumerate() {
            if c {
                self.forced |= 1 << i;
            }
        }
    }

    /// Fixed outcomes (the all-zero state, which seeding never produces):
    /// every coin is tails, every index 0, every shuffle the identity.
    /// Legality trials use it,
    /// so whether an option is legal never depends on chance.
    pub const fn zero() -> Rng {
        Rng { s: [0; 4], forced: 0, nforced: 0, rec: false }
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

    /// True for [`Rng::zero`]: a legality trial.
    #[inline]
    pub fn is_fixed(&self) -> bool {
        self.s == [0; 4]
    }

    pub fn coin(&mut self) -> bool {
        if self.is_fixed() {
            return false;
        }
        let v = self.coin_inner();
        record(self.rec, || Draw::Coin(v));
        v
    }

    fn coin_inner(&mut self) -> bool {
        if let Some(Draw::Coin(v)) = take_from_tape(|d| matches!(d, Draw::Coin(_))) {
            // The recording already holds a forced coin's result.
            if self.nforced > 0 {
                self.forced >>= 1;
                self.nforced -= 1;
            }
            return v;
        }
        if self.nforced > 0 {
            let v = self.forced & 1 == 1;
            self.forced >>= 1;
            self.nforced -= 1;
            return v;
        }
        self.below(2) == 0
    }

    /// Fisher–Yates permutation of `0..n`.
    pub fn shuffle(&mut self, n: usize, out: &mut [u8]) {
        for (i, o) in out.iter_mut().enumerate().take(n) {
            *o = i as u8;
        }
        if self.is_fixed() {
            return;
        }
        if let Some(Draw::Shuffle(v)) = take_from_tape(|d| matches!(d, Draw::Shuffle(v) if v.len() == n)) {
            out[..n].copy_from_slice(&v);
        } else {
            let mut i = n;
            while i > 1 {
                i -= 1;
                let j = self.below(i as u32 + 1) as usize;
                out.swap(i, j);
            }
        }
        record(self.rec, || Draw::Shuffle(out[..n].to_vec()));
    }

    pub fn index(&mut self, n: usize) -> usize {
        if self.is_fixed() {
            return self.below(n as u32) as usize;
        }
        let v = match take_from_tape(|d| matches!(d, Draw::Index(m, _) if *m == n)) {
            Some(Draw::Index(_, v)) => v,
            _ => self.below(n as u32) as usize,
        };
        record(self.rec, || Draw::Index(n, v));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_recorded_stream() {
        // Values produced by the TypeScript Rng(1): first shuffle of 60 from
        // the tier-1 trace (seed 1) starts 20,33,8,5,2,...
        let mut r = Rng::new(1);
        r.coin(); // setup: who begins
        let mut out = [0u8; 120];
        r.shuffle(60, &mut out);
        assert_eq!(&out[..5], &[20, 33, 8, 5, 2]);
    }

    /// The tape's counters: an outcome of the wanted kind and size is taken (left goes down), a draw the tape has nothing
    /// for is a miss, a legality trial's fixed generator never touches the tape, and clearing the tape clears both.
    #[test]
    fn the_tape_counts_what_it_could_not_serve() {
        set_tape(Some(vec![Draw::Coin(true), Draw::Shuffle(vec![1, 0]), Draw::Index(3, 2)]));
        assert_eq!((tape_left(), tape_misses()), (3, 0));
        let mut r = Rng::new(7);
        assert!(r.coin());
        assert_eq!(r.index(3), 2);
        assert_eq!((tape_left(), tape_misses()), (1, 0));
        let mut out = [0u8; 4];
        r.shuffle(3, &mut out);
        assert_eq!((tape_left(), tape_misses()), (1, 1), "no shuffle of 3 on the tape: a miss, the shuffle of 2 stays");
        r.coin();
        assert_eq!((tape_left(), tape_misses()), (1, 2));
        let mut z = Rng::zero();
        z.coin();
        z.index(3);
        z.shuffle(2, &mut out);
        assert_eq!((tape_left(), tape_misses()), (1, 2), "a fixed generator reads no tape");
        r.shuffle(2, &mut out);
        assert_eq!(&out[..2], &[1, 0]);
        assert_eq!((tape_left(), tape_misses()), (0, 2));
        set_tape(None);
        assert_eq!((tape_left(), tape_misses()), (0, 0));
        r.coin();
        assert_eq!(tape_misses(), 0, "no tape, no miss");
    }
}
