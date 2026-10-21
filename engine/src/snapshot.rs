//! Copies of a game: a restore into another game ([`Game::copy_from`]), a clone ([`Clone`] for [`Game`]), and a
//! [`Snapshot`] (the game's live parts in a buffer of their own size, for a search that stores many states).
//!
//! Most of `size_of::<Game>()` is unused stack capacity (prompts, effects, staging lists): every copy moves only the
//! live part, and the field list that says which part is live is written once ([`fields`]), so the copies, the snapshot
//! and the byte count ([`Game::copy_bytes`]) agree by construction. `Game` is not `Copy` (a whole-value copy would move
//! the capacity too).
//!
//! What a copy carries besides the fields:
//! - the declaration index (fixed per deal, rebuilt when a card's identity is rewritten): copied only into a game holding
//!   another one;
//! - the derived memo and the declaration order cache: not copied; the destination forgets its own (another line of play
//!   may reach the same counters);
//! - a restore (`copy_from`, [`Game::restore`]) keeps the destination's chance and event recording switches (restoring a
//!   recording game from a backup must not turn its recording off; the record goes back to the copy's entries); a clone
//!   copies them.

use crate::game::Game;
use crate::list::SVec;
use std::mem::MaybeUninit;
use std::ptr::addr_of as a;

/// What one way of copying a game does with each kind of field ([`fields`]). The projections take the game's address
/// and give a field's.
trait Mover {
    /// A field copied whole.
    unsafe fn whole<T: Copy>(&mut self, f: impl Fn(*const Game) -> *const T);
    /// A stack, by its live entries.
    unsafe fn live<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const SVec<T, N>);
    /// The first `n` entries of an array, `n` the value of the field `count` (copied before it).
    unsafe fn prefix<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const [T; N], count: impl Fn(*const Game) -> *const u8);
}

/// Every field of `Game` a copy moves, by kind (the index and the caches are the callers': see the module note).
#[inline(always)]
unsafe fn fields<M: Mover>(m: &mut M) {
    m.whole(|g| a!((*g).st));
    m.whole(|g| a!((*g).rng));
    m.live(|g| a!((*g).prompts));
    m.whole(|g| a!((*g).last_prompt_id));
    m.live(|g| a!((*g).items));
    m.live(|g| a!((*g).waits));
    m.live(|g| a!((*g).fx));
    // The staging lists ever allocated (not the 32 x 121 B of them).
    m.whole(|g| a!((*g).temp_hw));
    m.prefix(|g| a!((*g).temps), |g| a!((*g).temp_hw));
    m.whole(|g| a!((*g).temp_used));
    m.live(|g| a!((*g).coin_callbacks));
    m.whole(|g| a!((*g).trial));
    m.whole(|g| a!((*g).kinds_present));
    m.whole(|g| a!((*g).lock_cards));
    m.whole(|g| a!((*g).trace_effects));
    m.live(|g| a!((*g).copy_sessions));
    m.whole(|g| a!((*g).copy_serial));
    m.live(|g| a!((*g).copy_chain));
    m.whole(|g| a!((*g).deleg));
    m.live(|g| a!((*g).after_dmg));
    m.live(|g| a!((*g).pending));
    m.live(|g| a!((*g).ten_hp));
    m.live(|g| a!((*g).ten_hp_coin));
    m.whole(|g| a!((*g).last_attack));
    m.live(|g| a!((*g).spec_choices));
    m.live(|g| a!((*g).fxp.moves));
    m.live(|g| a!((*g).answers));
    m.live(|g| a!((*g).prompt_payloads));
    m.whole(|g| a!((*g).payload_serial));
    m.whole(|g| a!((*g).order_pending));
    m.whole(|g| a!((*g).checkup_held));
    m.whole(|g| a!((*g).put_by));
    m.whole(|g| a!((*g).prevented_events));
    m.whole(|g| a!((*g).reach_skip));
    m.whole(|g| a!((*g).survive_groups));
    m.whole(|g| a!((*g).record));
}

/// The field census: a new field of `Game` fails to compile here until [`fields`] (or the module note's list) has it.
const _: fn(&Game) = |g| {
    let Game {
        st: _, rng: _, prompts: _, last_prompt_id: _, items: _, waits: _, fx: _, temps: _, temp_used: _, temp_hw: _,
        coin_callbacks: _, trial: _, kinds_present: _, lock_cards: _, trace_effects: _, copy_sessions: _, copy_serial: _,
        copy_chain: _, deleg: _, after_dmg: _, pending: _, ten_hp: _, ten_hp_coin: _, last_attack: _, spec_choices: _,
        derived: _, decls: _, order_cache: _, fxp: crate::effects::EffectPayloads { moves: _ }, answers: _,
        prompt_payloads: _, payload_serial: _, order_pending: _, checkup_held: _, put_by: _, prevented_events: _,
        reach_skip: _, survive_groups: _, record: _,
    } = g;
};

/// Game to game: the fields of `src` into `dst`. `fresh`: `dst` is uninitialized memory, so the unused staging lists are
/// written too (from `src`: no `Game` value may hold uninitialized lists).
struct ToGame {
    src: *const Game,
    dst: *mut Game,
    fresh: bool,
}

impl Mover for ToGame {
    #[inline(always)]
    unsafe fn whole<T: Copy>(&mut self, f: impl Fn(*const Game) -> *const T) {
        (f(self.dst) as *mut T).write(*f(self.src));
    }
    #[inline(always)]
    unsafe fn live<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const SVec<T, N>) {
        (*f(self.src)).copy_live_to(f(self.dst) as *mut SVec<T, N>);
    }
    #[inline(always)]
    unsafe fn prefix<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const [T; N], count: impl Fn(*const Game) -> *const u8) {
        let n = if self.fresh { N } else { *count(self.src) as usize };
        std::ptr::copy_nonoverlapping(f(self.src) as *const T, f(self.dst) as *mut T, n);
    }
}

/// The bytes a copy moves (`Game::copy_bytes`).
struct Count {
    src: *const Game,
    bytes: usize,
}

impl Mover for Count {
    #[inline(always)]
    unsafe fn whole<T: Copy>(&mut self, _: impl Fn(*const Game) -> *const T) {
        self.bytes += std::mem::size_of::<T>();
    }
    #[inline(always)]
    unsafe fn live<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const SVec<T, N>) {
        self.bytes += (*f(self.src)).live_bytes();
    }
    #[inline(always)]
    unsafe fn prefix<T: Copy, const N: usize>(&mut self, _: impl Fn(*const Game) -> *const [T; N], count: impl Fn(*const Game) -> *const u8) {
        self.bytes += *count(self.src) as usize * std::mem::size_of::<T>();
    }
}

/// Game to snapshot: the fields of `src` appended to `out` as raw bytes (padding included, hence `MaybeUninit`).
struct Write<'a> {
    src: *const Game,
    out: &'a mut Vec<MaybeUninit<u8>>,
}

impl Write<'_> {
    #[inline(always)]
    unsafe fn put(&mut self, p: *const u8, n: usize) {
        let len = self.out.len();
        self.out.reserve(n);
        std::ptr::copy_nonoverlapping(p as *const MaybeUninit<u8>, self.out.as_mut_ptr().add(len), n);
        self.out.set_len(len + n);
    }
}

impl Mover for Write<'_> {
    #[inline(always)]
    unsafe fn whole<T: Copy>(&mut self, f: impl Fn(*const Game) -> *const T) {
        self.put(f(self.src) as *const u8, std::mem::size_of::<T>());
    }
    #[inline(always)]
    unsafe fn live<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const SVec<T, N>) {
        let (len, items) = SVec::raw_parts(f(self.src));
        self.put(len, 1);
        self.put(items as *const u8, *len as usize * std::mem::size_of::<T>());
    }
    #[inline(always)]
    unsafe fn prefix<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const [T; N], count: impl Fn(*const Game) -> *const u8) {
        self.put(f(self.src) as *const u8, *count(self.src) as usize * std::mem::size_of::<T>());
    }
}

/// Snapshot to game: the bytes [`Write`] appended, back into the fields of `dst` in the same order.
struct Read<'a> {
    bytes: &'a [MaybeUninit<u8>],
    pos: usize,
    dst: *mut Game,
}

impl Read<'_> {
    #[inline(always)]
    unsafe fn take(&mut self, p: *mut u8, n: usize) {
        let src = &self.bytes[self.pos..self.pos + n];
        std::ptr::copy_nonoverlapping(src.as_ptr(), p as *mut MaybeUninit<u8>, n);
        self.pos += n;
    }
}

impl Mover for Read<'_> {
    #[inline(always)]
    unsafe fn whole<T: Copy>(&mut self, f: impl Fn(*const Game) -> *const T) {
        self.take(f(self.dst) as *mut u8, std::mem::size_of::<T>());
    }
    #[inline(always)]
    unsafe fn live<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const SVec<T, N>) {
        let (len, items) = SVec::raw_parts(f(self.dst));
        self.take(len as *mut u8, 1);
        let n = *len as usize;
        assert!(n <= N, "snapshot: a stack past its capacity");
        self.take(items as *mut u8, n * std::mem::size_of::<T>());
    }
    #[inline(always)]
    unsafe fn prefix<T: Copy, const N: usize>(&mut self, f: impl Fn(*const Game) -> *const [T; N], count: impl Fn(*const Game) -> *const u8) {
        // The count is a field restored before the array.
        let n = *count(self.dst) as usize;
        assert!(n <= N, "snapshot: an array past its length");
        self.take(f(self.dst) as *mut u8, n * std::mem::size_of::<T>());
    }
}

/// A game's live parts in a buffer of their size (about 10 KB against `size_of::<Game>()`'s 64 KB), for a search that
/// keeps many states: [`Game::snapshot`] / [`Game::snapshot_into`] take one, [`Game::restore`] puts it back into a game
/// as [`Game::copy_from`] of the game it was taken from would. Plain data (`Send`); it holds its game's declaration index,
/// so it can be restored into any game (one whose cards were rewritten since included).
#[derive(Clone)]
pub struct Snapshot {
    decls: crate::decl::DeclIndex,
    bytes: Vec<MaybeUninit<u8>>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Snapshot { decls: crate::decl::DeclIndex::new(), bytes: Vec::new() }
    }
}

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Snapshot({} B)", self.size())
    }
}

impl Snapshot {
    /// The bytes the snapshot holds: its buffer's live part and its declaration index.
    pub fn size(&self) -> usize {
        self.bytes.len() + std::mem::size_of::<crate::decl::DeclIndex>()
    }
}

/// What [`Game::copy_from`] moves: a restore's memory traffic besides the work after it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CopyBytes {
    /// Copied from the source: the state, the live part of the stacks, the staging lists ever allocated, the fixed
    /// fields (not the declaration index: a restore into a game of the same deal holds it already).
    pub copied: usize,
    /// Written in the destination to forget its derived facts (the memo's stamps).
    pub cleared: usize,
}

impl Game {
    /// Overwrite `self` with `src`, moving only the live parts (the module note says what else moves): the game a search
    /// restores into plays on as `src` would.
    #[inline]
    pub fn copy_from(&mut self, src: &Game) {
        let (rec, on) = (self.rng.recording(), self.record.on);
        // SAFETY: `self` is a valid, exclusive Game and `src` a valid one (not `self`: the borrows exclude it); the stack
        // tails are `MaybeUninit`.
        unsafe { fields(&mut ToGame { src, dst: self, fresh: false }) };
        if self.decls.id != src.decls.id {
            self.decls = src.decls;
        }
        self.restored(rec, on);
    }

    /// The live parts of the game, in a new [`Snapshot`].
    pub fn snapshot(&self) -> Snapshot {
        let mut s = Snapshot::default();
        self.snapshot_into(&mut s);
        s
    }

    /// The live parts of the game into `s`, its buffer reused (no allocation once it has grown to the game's size).
    pub fn snapshot_into(&self, s: &mut Snapshot) {
        s.bytes.clear();
        s.decls = self.decls;
        // SAFETY: `self` is a valid Game; the writer only reads it.
        unsafe { fields(&mut Write { src: self, out: &mut s.bytes }) };
    }

    /// Overwrite `self` with the game `s` was taken from, as [`copy_from`](Self::copy_from) of that game would.
    pub fn restore(&mut self, s: &Snapshot) {
        let (rec, on) = (self.rng.recording(), self.record.on);
        let mut r = Read { bytes: &s.bytes, pos: 0, dst: self };
        // SAFETY: `self` is a valid, exclusive Game; the bytes are a `Write` of a valid Game, read back field by field in
        // the order they were written, each stack length checked against its capacity.
        unsafe { fields(&mut r) };
        assert_eq!(r.pos, s.bytes.len(), "snapshot: bytes left over");
        if self.decls.id != s.decls.id {
            self.decls = s.decls;
        }
        self.restored(rec, on);
    }

    /// After a restore: the destination's recording switches back (`rec` the chance recording; `on` the game record,
    /// whose buffer goes back to the copy's entries: a rolled back action's go) and its caches forgotten.
    #[inline]
    fn restored(&mut self, rec: bool, on: bool) {
        self.rng.set_record(rec);
        self.record = self.record.with_on(on);
        if on {
            crate::record::rewind(self.record.len);
        }
        self.derived.clear();
        self.order_cache.invalidate();
    }

    /// A clone of the game in a new box, written in place (no whole-`Game` copy on the way).
    pub fn boxed(&self) -> Box<Game> {
        let mut b = Box::<Game>::new_uninit();
        // SAFETY: the box is valid for writes; `clone_to` with `fresh` writes every field.
        unsafe {
            self.clone_to(b.as_mut_ptr(), true);
            b.assume_init()
        }
    }

    /// Write a clone of `self` to `d`: every field (`fresh`: `d` is uninitialized memory) or the live parts over a valid
    /// game. The recording switches are copied; the caches start empty.
    #[inline]
    unsafe fn clone_to(&self, d: *mut Game, fresh: bool) {
        use std::ptr::addr_of_mut as m;
        fields(&mut ToGame { src: self, dst: d, fresh });
        if fresh {
            m!((*d).decls).write(self.decls);
            m!((*d).derived).write(crate::derived::Derived::new());
            m!((*d).order_cache).write(crate::decl::OrderCache::new());
        } else {
            if (*d).decls.id != self.decls.id {
                m!((*d).decls).write(self.decls);
            }
            (*d).derived.clear();
            (*d).order_cache.invalidate();
        }
    }

    /// What [`copy_from`](Self::copy_from) moves when `self` is the source (a measurement: `bench --search`,
    /// tests/sizes.rs). `#[inline]`: compiled only where it is called.
    #[inline]
    pub fn copy_bytes(&self) -> CopyBytes {
        let mut c = Count { src: self, bytes: 0 };
        // SAFETY: the counter only reads `self`.
        unsafe { fields(&mut c) };
        CopyBytes { copied: c.bytes, cleared: self.derived.clear_bytes() }
    }
}

/// A clone moves the live parts (most of `size_of::<Game>()` is unused capacity: `Game` is not `Copy`); its caches
/// start empty. `clone_from` writes over the destination in place.
impl Clone for Game {
    #[inline]
    fn clone(&self) -> Game {
        let mut g = MaybeUninit::<Game>::uninit();
        // SAFETY: `clone_to` with `fresh` writes every field.
        unsafe {
            self.clone_to(g.as_mut_ptr(), true);
            g.assume_init()
        }
    }

    #[inline]
    fn clone_from(&mut self, src: &Game) {
        // SAFETY: `self` is a valid, exclusive Game.
        unsafe { src.clone_to(self, false) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{self, Decision};
    use crate::rng::Rng;

    fn decks() -> Vec<Vec<crate::carddb::DefId>> {
        let spec: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/bench/spec.json")).unwrap()).unwrap();
        spec["decks"].as_array().unwrap().iter().map(|d| crate::selfplay::expand_deck(&d["cards"]).unwrap()).collect()
    }

    /// One decision of `g` played at random (a turn action, or picks from the pick masks); `false` when the game is over
    /// or the random answer is refused.
    fn step(g: &mut Game, rng: &mut Rng) -> bool {
        match host::next(g).unwrap() {
            Decision::Over { .. } => false,
            Decision::Turn { actions, .. } => g.act(actions[rng.index(actions.len())]).is_ok(),
            Decision::Prompt { select, .. } => {
                let mut picks = Vec::new();
                loop {
                    let (mask, done) = g.pick_mask(&select, &picks);
                    let allowed: Vec<usize> = (0..mask.len()).filter(|j| mask[*j]).collect();
                    let r = rng.index(allowed.len() + done as usize);
                    if r == allowed.len() {
                        break;
                    }
                    picks.push(allowed[r]);
                    if select.max_count <= 1 {
                        break;
                    }
                }
                g.answer(&select, &picks).is_ok()
            }
        }
    }

    /// `k` decisions of `g` with the policy stream `seed`; the state after them.
    fn play_on(g: &mut Game, seed: u32, k: usize) -> String {
        let mut rng = Rng::new(seed);
        for _ in 0..k {
            if !step(g, &mut rng) {
                break;
            }
        }
        g.state_hash()
    }

    /// At decisions of random games (prompts in flight included), a clone, a boxed clone, a restore into a game of
    /// another deal, a restore from a snapshot into a game of another deal and `clone_from` over a fork are the game: the
    /// same state, and the same state after the same play on. A snapshot holds exactly the bytes `copy_from` moves.
    #[test]
    fn copies_play_on_as_the_game() {
        let decks = decks();
        let mut checked = 0;
        for seed in 0..4u32 {
            let d = |i: u32| decks[i as usize % decks.len()].as_slice();
            let mut g = host::start_game([d(seed), d(seed + 1)], seed, &host::StartOptions::default()).unwrap();
            let other = host::start_game([d(seed + 2), d(seed + 3)], seed + 50, &host::StartOptions::default()).unwrap();
            let mut rng = Rng::new(seed + 100);
            for n in 0..400u32 {
                if n % 9 == 0 {
                    let s = g.snapshot();
                    assert_eq!(s.bytes.len(), g.copy_bytes().copied);
                    let mut clone = g.clone();
                    let mut boxed = g.boxed();
                    let mut restored = other.boxed();
                    restored.copy_from(&g);
                    let mut from_snap = other.boxed();
                    from_snap.restore(&s);
                    assert_eq!(from_snap.decls.id, g.decls.id);
                    let mut over = g.fork();
                    over.clone_from(&other);
                    over.clone_from(&g);
                    let want = play_on(&mut g.boxed(), n, 6);
                    for (name, c) in [("clone", &mut clone), ("boxed", &mut *boxed), ("copy_from", &mut *restored), ("snapshot", &mut *from_snap), ("clone_from", &mut *over)] {
                        assert_eq!(c.state_hash(), g.state_hash(), "{name} at decision {n} of game {seed}");
                        assert_eq!(play_on(c, n, 6), want, "{name} played on, decision {n} of game {seed}");
                    }
                    checked += 1;
                }
                if !step(&mut g, &mut rng) {
                    break;
                }
            }
        }
        assert!(checked > 40, "{checked} decisions checked");
    }
}
