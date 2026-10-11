//! The derived layer (events design section 6; batch 8 plan section 2): the facts continuous effects decide, per Pokémon
//! slot, per card and per player, read by execution and legality alike through one read path.
//!
//! **One read path (batch 8 C2).** Every read of a fact goes through a function here: HP ([`hp`]), types ([`types`]),
//! Weakness and Resistance ([`weakness_resistance`]), the Energy provided ([`provided_energy`]), the Retreat Cost
//! ([`retreat_cost`]), the attack list ([`attacks`]), an attack's cost ([`attack_cost`]), the powers a Pokémon has
//! ([`powers`]), "has no Ability" ([`has_no_ability`]), the Special Conditions kept by evolving
//! ([`preserved_conditions`]), and the lock / prevention queries ([`event_locked`], [`event_prevented`]). The facts are
//! computed by `compute_*` (C2: today's check-effect runs, kept as `slow_*` until the closeout) and memoized against
//! `State::version`.
//!
//! **Invalidation.** A fact is fresh while its stamp equals `State::version`, which every write bumps (the event routines
//! at their start and end and before their reducers, the physical moves, every interpreter op step and continuation, every
//! action and answer, the lock stamps' changes; `state.rs`). The memo lives in the `Game` next to the state, so a copy
//! made by assignment carries consistent pairs; `Game::copy_from` (forks, the rollback) clears it, since another line of
//! play may reach the same version numbers. `PTCG_VERIFY_CACHE=1` recomputes every memo hit through the slow path and
//! compares, and checks a digest of the state taken at the first store of each version against a missed bump.

use crate::effects::{Cost, Effect, EnergyMap, PowerRef, ResistanceV, SlotRef, WeaknessV};
use crate::game::{Game, R};
use crate::list::*;
use crate::state::{AttackRef, MAX_CARDS, MAX_SLOTS};
use crate::types::CardType;

/// Entries of the attack-cost memo (the attacks of the Active Pokémon asked within one version).
const COST_MEMO: usize = 4;

/// Per game: the memo of the derived facts, each with the `State::version` it was computed at (0: none).
#[derive(Clone, Copy)]
pub struct Derived {
    hp_v: [[u32; MAX_SLOTS]; 2],
    hp: [[i32; MAX_SLOTS]; 2],
    types_v: [[u32; MAX_SLOTS]; 2],
    types: [[SVec<CardType, 4>; MAX_SLOTS]; 2],
    /// Provided Energy per source slot, with the player who asked (`CheckProvidedEnergy::p`).
    pe_v: [[u32; MAX_SLOTS]; 2],
    pe_p: [[u8; MAX_SLOTS]; 2],
    pe: [[EnergyMap; MAX_SLOTS]; 2],
    retreat_v: [u32; 2],
    retreat: [Cost; 2],
    attacks_v: [u32; 2],
    attacks: [crate::engine::turn::CheckedAttacks; 2],
    cost_v: [u32; COST_MEMO],
    cost_key: [(u8, AttackRef); COST_MEMO],
    cost: [Cost; COST_MEMO],
    cost_next: u8,
    /// The powers a Pokémon card has now (`CheckPokemonPowers`), with the player who asked.
    powers_v: [u32; MAX_CARDS],
    powers_p: [u8; MAX_CARDS],
    powers: [SVec<PowerRef, 8>; MAX_CARDS],
    /// "Has no Ability" per card: bit i for power i, bit 7 for the generic probe; `abil_known` the bits computed at
    /// `abil_v` for the player `abil_p`, `abil_off` the answers.
    abil_v: [u32; MAX_CARDS],
    abil_p: [u8; MAX_CARDS],
    abil_known: [u8; MAX_CARDS],
    abil_off: [u8; MAX_CARDS],
    /// `PTCG_VERIFY_CACHE=1`: the digest of the state at the first store of version `dig_v`.
    dig_v: u32,
    dig: u64,
}

impl Derived {
    pub fn new() -> Derived {
        Derived {
            hp_v: [[0; MAX_SLOTS]; 2],
            hp: [[0; MAX_SLOTS]; 2],
            types_v: [[0; MAX_SLOTS]; 2],
            types: [[SVec::new(); MAX_SLOTS]; 2],
            pe_v: [[0; MAX_SLOTS]; 2],
            pe_p: [[0; MAX_SLOTS]; 2],
            pe: [[SVec::new(); MAX_SLOTS]; 2],
            retreat_v: [0; 2],
            retreat: [SVec::new(); 2],
            attacks_v: [0; 2],
            attacks: [(SVec::new(), SVec::new()); 2],
            cost_v: [0; COST_MEMO],
            cost_key: [(0, AttackRef { card: 0, index: 0 }); COST_MEMO],
            cost: [SVec::new(); COST_MEMO],
            cost_next: 0,
            powers_v: [0; MAX_CARDS],
            powers_p: [0; MAX_CARDS],
            powers: [SVec::new(); MAX_CARDS],
            abil_v: [0; MAX_CARDS],
            abil_p: [0; MAX_CARDS],
            abil_known: [0; MAX_CARDS],
            abil_off: [0; MAX_CARDS],
            dig_v: 0,
            dig: 0,
        }
    }

    /// Forget every fact (the stamps only): the game was overwritten by another line of play.
    #[inline]
    pub fn clear(&mut self) {
        self.hp_v = [[0; MAX_SLOTS]; 2];
        self.types_v = [[0; MAX_SLOTS]; 2];
        self.pe_v = [[0; MAX_SLOTS]; 2];
        self.retreat_v = [0; 2];
        self.attacks_v = [0; 2];
        self.cost_v = [0; COST_MEMO];
        self.powers_v = [0; MAX_CARDS];
        self.abil_v = [0; MAX_CARDS];
        self.dig_v = 0;
    }
}

impl Default for Derived {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// VERIFY: the memo against a recompute, the state against a missed version bump

/// A store at the game's version: under VERIFY, the digest of the state at the first store of the version.
#[inline]
fn stored(g: &mut Game) {
    if crate::game::verify_cache() && g.derived.dig_v != g.st.version {
        g.derived.dig_v = g.st.version;
        g.derived.dig = state_digest(&g.st);
    }
}

/// `PTCG_VERIFY_CACHE=1`: no write since the last store at this version (checked where routines that write outside an
/// event hand over to readers: the entry of the state check).
#[inline]
pub fn verify_version(g: &Game, at: &str) {
    if crate::game::verify_cache() && g.derived.dig_v == g.st.version {
        assert_eq!(g.derived.dig, state_digest(&g.st), "{at}: the state was written without a version bump (version {})", g.st.version);
    }
}

/// A memo hit: under VERIFY, the state is the one the fact was computed in (same digest) and the recompute agrees.
#[inline]
fn hit<T: PartialEq + std::fmt::Debug>(g: &mut Game, what: &str, memo: T, again: impl FnOnce(&mut Game) -> T) -> T {
    if crate::game::verify_cache() {
        assert_eq!(g.derived.dig_v, g.st.version, "derived::{what}: a memo hit with no digest of its version");
        assert_eq!(g.derived.dig, state_digest(&g.st), "derived::{what}: the state was written without a version bump (version {})", g.st.version);
        let fresh = again(g);
        assert_eq!(memo, fresh, "derived::{what}: the memo differs from a recompute");
    }
    memo
}

/// A digest of everything a fact can read in the state (`PTCG_VERIFY_CACHE=1` only): the zones, the spots and their
/// records, the players' records, the cards' instance fields, the globals. Not `Slot::hp_bonus` (written by the HP read
/// itself until it is computed directly), nor the version counters and the location hints.
pub fn state_digest(st: &crate::state::State) -> u64 {
    use std::hash::{Hash, Hasher};
    struct H(u64);
    impl Hasher for H {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, bytes: &[u8]) {
            for b in bytes {
                self.write_u64(*b as u64);
            }
        }
        fn write_u64(&mut self, x: u64) {
            self.0 = (self.0.rotate_left(5) ^ x).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
        fn write_u8(&mut self, x: u8) {
            self.write_u64(x as u64);
        }
        fn write_u16(&mut self, x: u16) {
            self.write_u64(x as u64);
        }
        fn write_u32(&mut self, x: u32) {
            self.write_u64(x as u64);
        }
        fn write_i32(&mut self, x: i32) {
            self.write_u64(x as u32 as u64);
        }
        fn write_usize(&mut self, x: usize) {
            self.write_u64(x as u64);
        }
    }
    let mut h = H(0);
    let l = |h: &mut H, s: &[CardId]| {
        h.write_usize(s.len());
        for c in s {
            h.write_u8(*c);
        }
    };
    let ar = |h: &mut H, a: Option<AttackRef>| match a {
        Some(a) => {
            h.write_u8(1);
            h.write_u8(a.card);
            h.write_u8(a.index);
        }
        None => h.write_u8(0),
    };
    let cause = |h: &mut H, c: &crate::cause::Cause| {
        c.kind.hash(h);
        c.card.hash(h);
        h.write_u8(c.player);
        match c.attack {
            Some(a) => {
                h.write_u8(a.card);
                h.write_u8(a.index);
            }
            None => h.write_u8(0xFF),
        }
    };
    h.write_u8(st.phase as u8);
    h.write_i32(st.turn);
    h.write_u8(st.active_player);
    h.write_u8(st.winner as u8);
    std::mem::discriminant(&st.end_reason).hash(&mut h);
    h.write_u8(st.n_cards);
    let r = &st.rules;
    for b in [r.first_turn_draw_card, r.first_turn_use_supporter, r.attack_first_turn, r.supporter_cleanup_at_end_turn, r.unlimited_energy_attachments, st.skip_opponent_turn, st.is_sudden_death, st.bench_size_change_handled] {
        h.write_u8(b as u8);
    }
    ar(&mut h, st.last_attack);
    for p in 0..2 {
        match st.player_last_attack[p] {
            Some((a, c)) => {
                ar(&mut h, Some(a));
                h.write_u8(c);
            }
            None => h.write_u8(0),
        }
        h.write_i32(st.player_last_attack_turn[p]);
    }
    h.write_u8(st.players_added);
    h.write_u16(st.ability_lock_order_counter);
    for c in 0..st.n_cards as usize {
        let ci = &st.cards[c];
        h.write_u16(ci.def);
        h.write_u8(ci.owner);
        h.write_u8(ci.moved_to_active_this_turn as u8);
        h.write_i32(ci.damage_taken_last_turn);
        for b in [ci.extra_prizes as u8, ci.attack_barrage, ci.attack_barrage_shown, ci.attack_shred, ci.strafe_used as u8, ci.discarded_stadium_card as u8] {
            h.write_u8(b);
        }
        h.write_usize(ci.evolves_from_base.map_or(0, |s| s.as_ptr() as usize));
        h.write_u16(ci.lock_stamp);
        std::mem::discriminant(&ci.staged_from).hash(&mut h);
        if let Some(z) = ci.staged_from {
            std::mem::discriminant(&z).hash(&mut h);
        }
        ci.staged_in.hash(&mut h);
    }
    for pl in st.players.iter() {
        h.write_u8(pl.id);
        for z in [&pl.deck, &pl.hand, &pl.discard, &pl.lostzone] {
            l(&mut h, z.as_slice());
        }
        l(&mut h, pl.stadium.as_slice());
        l(&mut h, pl.supporter.as_slice());
        h.write_u8(pl.prize_count);
        for i in 0..6 {
            l(&mut h, pl.prizes[i].as_slice());
            h.write_u8(pl.prize_public[i] as u8);
            h.write_u8(pl.prize_face_up[i] as u8);
        }
        h.write_u8(pl.active);
        l(&mut h, pl.bench.as_slice());
        for s in 0..MAX_SLOTS {
            h.write_u8(pl.slot_used[s] as u8);
            let sl = &pl.slots[s];
            l(&mut h, sl.cards.as_slice());
            l(&mut h, sl.energies.as_slice());
            l(&mut h, sl.tools.as_slice());
            for x in [sl.damage, sl.hp, sl.poison_damage, sl.burn_damage, sl.confusion_damage, sl.entered_turn, sl.sleep_flips] {
                h.write_i32(x);
            }
            l(&mut h, sl.special_conditions.as_slice());
            l(&mut h, sl.board_effect.as_slice());
            for m in sl.marker.items.iter() {
                h.write_u16(m.name);
                h.write_u8(m.source);
                h.write_u8(m.source_type as u8);
                h.write_u8(m.target_scope as u8);
            }
            sl.attacks_this_turn.hash(&mut h);
            h.write_u8(sl.healed_this_turn as u8);
            for lk in sl.lasting_locks.iter() {
                h.write_usize(lk.decl as *const _ as usize);
                h.write_u8(lk.turns_remaining as u8);
                h.write_u8(lk.source);
                h.write_usize(lk.attack.map_or(0, |a| a.as_ptr() as usize));
            }
            for x in [sl.damage_reduction_next_turn, sl.attack_damage_reduction_next_turn, sl.defending_extra_damage_next_turn, sl.attack_cost_increase_next_turn, sl.attack_cost_increase_next_turn_pending, sl.retreat_cost_increase_next_turn, sl.retreat_cost_increase_next_turn_pending] {
                h.write_i32(x);
            }
            for b in [sl.discard_attacker_energy_if_ko_next_turn, sl.discard_attacker_energy_if_ko_next_turn_pending, sl.no_weakness_next_turn, sl.no_weakness_next_turn_pending, sl.defending_extra_damage_pending, sl.defending_extra_damage_rearm_after_attack, sl.is_public] {
                h.write_u8(b as u8);
            }
            ar(&mut h, sl.discard_attacker_energy_if_ko_attack);
            sl.discard_attacker_energy_if_ko_source_card.hash(&mut h);
            sl.discard_attacker_energy_if_ko_attacker.hash(&mut h);
            for b in [sl.next_turn_attack_damage_bonus, sl.next_turn_attack_damage_bonus_pending] {
                match b {
                    Some(x) => {
                        h.write_usize(x.attack_name.as_ptr() as usize);
                        h.write_i32(x.bonus_damage);
                        h.write_usize(x.source_card_name.as_ptr() as usize);
                    }
                    None => h.write_u8(0),
                }
            }
            sl.defending_extra_damage_attacker.hash(&mut h);
            sl.attack_cost_increase_next_turn_attacker.hash(&mut h);
            sl.retreat_cost_increase_next_turn_attacker.hash(&mut h);
            for r in [sl.retaliate_on_damage_next_turn, sl.retaliate_on_damage_next_turn_pending] {
                match r {
                    Some(x) => {
                        h.write_i32(x.damage);
                        ar(&mut h, Some(x.attack));
                        h.write_u8(x.source_card);
                        h.write_u8(x.attacker);
                    }
                    None => h.write_u8(0),
                }
            }
            for lp in sl.lasting_prevents.iter() {
                h.write_u8(lp.spec);
                h.write_u8(lp.source);
                h.write_u8(lp.pending as u8);
            }
            match &sl.ko_pending {
                Some(c) => cause(&mut h, c),
                None => h.write_u8(0),
            }
        }
        l(&mut h, pl.played_this_turn.as_slice());
        for x in [pl.retreated_turn, pl.energy_played_turn, pl.stadium_used_turn, pl.prizes_taken, pl.prizes_taken_this_turn, pl.prizes_taken_last_turn, pl.unlimited_energy_attach_turns_remaining, pl.stadium_and_tool_have_no_effect_turns_remaining] {
            h.write_i32(x);
        }
        for m in pl.marker.items.iter() {
            h.write_u16(m.name);
            h.write_u8(m.source);
            h.write_u8(m.source_type as u8);
            h.write_u8(m.target_scope as u8);
        }
        for b in [pl.used_vstar, pl.used_gx, pl.pokemon_knocked_out_during_opponents_last_turn, pl.pokemon_knocked_out_by_attack_during_opponents_last_turn, pl.can_evolve, pl.ancient_pokemon_attacked_last_turn, pl.used_dragons_wish, pl.legacy_energy_used] {
            h.write_u8(b as u8);
        }
        l(&mut h, pl.moved_to_active_this_turn.as_slice());
        l(&mut h, pl.moved_from_active_to_bench_this_turn.as_slice());
        for d in pl.pokemon_knocked_out_last_turn_entries.iter() {
            h.write_u16(*d);
        }
        for b in pl.pokemon_knocked_out_last_turn_by_attack.iter() {
            h.write_u8(*b as u8);
        }
        for lk in pl.lasting_locks.iter() {
            match lk {
                Some(x) => {
                    h.write_usize(x.decl as *const _ as usize);
                    h.write_u8(x.turns_remaining as u8);
                    h.write_u8(x.source);
                    h.write_usize(x.attack.map_or(0, |a| a.as_ptr() as usize));
                }
                None => h.write_u8(0),
            }
        }
    }
    h.finish()
}

// ---------------------------------------------------------------------------
// The reads

/// The HP of the Pokémon in slot `s` of player `p` (every HP modifier applied).
#[inline]
pub fn hp(g: &mut Game, p: usize, s: crate::state::SlotId) -> R<i32> {
    let v = g.st.version;
    if g.derived.hp_v[p][s as usize] == v {
        let m = g.derived.hp[p][s as usize];
        return Ok(hit(g, "hp", m, |g| game_hp(g, p, s).unwrap_or(i32::MIN)));
    }
    let r = game_hp(g, p, s)?;
    if g.st.version == v {
        stored(g);
        g.derived.hp_v[p][s as usize] = v;
        g.derived.hp[p][s as usize] = r;
    }
    Ok(r)
}

/// The Energy the Pokémon in `source` provides, per attached card (asked by `p`; APR C-01).
#[inline]
pub fn provided_energy(g: &mut Game, p: usize, source: SlotRef) -> R<EnergyMap> {
    let v = g.st.version;
    let (q, s) = (source.p as usize, source.s as usize);
    if g.derived.pe_v[q][s] == v && g.derived.pe_p[q][s] == p as u8 {
        let m = g.derived.pe[q][s];
        return Ok(hit(g, "provided_energy", m, |g| game_provided_energy(g, p, source).unwrap_or_default()));
    }
    let r = game_provided_energy(g, p, source)?;
    if g.st.version == v {
        stored(g);
        g.derived.pe_v[q][s] = v;
        g.derived.pe_p[q][s] = p as u8;
        g.derived.pe[q][s] = r;
    }
    Ok(r)
}

/// The types of the Pokémon in `target`, as the game checks them (`TypeOverride` and the like applied).
#[inline]
pub fn types(g: &mut Game, target: SlotRef) -> R<SVec<CardType, 4>> {
    let v = g.st.version;
    let (q, s) = (target.p as usize, target.s as usize);
    if g.derived.types_v[q][s] == v {
        let m = g.derived.types[q][s];
        return Ok(hit(g, "types", m, |g| game_types(g, target).unwrap_or_default()));
    }
    let r = game_types(g, target)?;
    if g.st.version == v {
        stored(g);
        g.derived.types_v[q][s] = v;
        g.derived.types[q][s] = r;
    }
    Ok(r)
}

/// The Weakness and Resistance of the Pokémon in `target` (read by the damage calculation only: not memoized).
pub fn weakness_resistance(g: &mut Game, target: SlotRef) -> R<(SVec<WeaknessV, 4>, SVec<ResistanceV, 4>)> {
    game_weakness_resistance(g, target)
}

/// Player `p`'s Active Pokémon's Retreat Cost.
#[inline]
pub fn retreat_cost(g: &mut Game, p: usize) -> R<Cost> {
    let v = g.st.version;
    if g.derived.retreat_v[p] == v {
        let m = g.derived.retreat[p];
        return Ok(hit(g, "retreat_cost", m, |g| game_retreat_cost(g, p).unwrap_or_default()));
    }
    let r = game_retreat_cost(g, p)?;
    if g.st.version == v {
        stored(g);
        g.derived.retreat_v[p] = v;
        g.derived.retreat[p] = r;
    }
    Ok(r)
}

/// The attacks player `p`'s Active Pokémon may use, and those copied from the Bench.
#[inline]
pub fn attacks(g: &mut Game, p: usize) -> R<crate::engine::turn::CheckedAttacks> {
    let v = g.st.version;
    if g.derived.attacks_v[p] == v {
        let m = g.derived.attacks[p];
        return Ok(hit(g, "attacks", m, |g| game_attacks(g, p).unwrap_or_default()));
    }
    let r = game_attacks(g, p)?;
    if g.st.version == v {
        stored(g);
        g.derived.attacks_v[p] = v;
        g.derived.attacks[p] = r;
    }
    Ok(r)
}

/// The current cost of `attack` (asked by `p`, the attacking player).
#[inline]
pub fn attack_cost(g: &mut Game, p: usize, attack: AttackRef) -> R<Cost> {
    let v = g.st.version;
    for i in 0..COST_MEMO {
        if g.derived.cost_v[i] == v && g.derived.cost_key[i] == (p as u8, attack) {
            let m = g.derived.cost[i];
            return Ok(hit(g, "attack_cost", m, |g| game_attack_cost(g, p, attack).unwrap_or_default()));
        }
    }
    let r = game_attack_cost(g, p, attack)?;
    if g.st.version == v {
        stored(g);
        let i = g.derived.cost_next as usize % COST_MEMO;
        g.derived.cost_next = g.derived.cost_next.wrapping_add(1);
        g.derived.cost_v[i] = v;
        g.derived.cost_key[i] = (p as u8, attack);
        g.derived.cost[i] = r;
    }
    Ok(r)
}

/// The powers the Pokémon card `card` has now (asked by `p`): its printed ones, less those an Ability lock removes.
#[inline]
pub fn powers(g: &mut Game, p: usize, card: CardId) -> R<SVec<PowerRef, 8>> {
    let v = g.st.version;
    let c = card as usize;
    if g.derived.powers_v[c] == v && g.derived.powers_p[c] == p as u8 {
        let m = g.derived.powers[c];
        return Ok(hit(g, "powers", m, |g| game_powers(g, p, card).unwrap_or_default()));
    }
    let r = game_powers(g, p, card)?;
    if g.st.version == v {
        stored(g);
        g.derived.powers_v[c] = v;
        g.derived.powers_p[c] = p as u8;
        g.derived.powers[c] = r;
    }
    Ok(r)
}

/// Does `card` (player `p`'s) have no Ability (power `power_index`, or the generic probe) now? (The lock pass with
/// take-hold stamps; RULES.md "Ability locks".) Not memoized while a copied attack's source code runs (its copycat has
/// no Ability of its own); a stamp changed by `lock_sync` is a write (a version bump).
#[inline]
pub fn has_no_ability(g: &mut Game, p: usize, card: CardId, power_index: Option<u8>) -> bool {
    if g.deleg.is_some() || card as usize >= MAX_CARDS {
        return game_has_no_ability(g, p, card, power_index);
    }
    let bit = match power_index {
        Some(i) if i < 7 => 1u8 << i,
        Some(_) => return game_has_no_ability(g, p, card, power_index),
        None => 1u8 << 7,
    };
    let v = g.st.version;
    let c = card as usize;
    if g.derived.abil_v[c] == v && g.derived.abil_p[c] == p as u8 {
        if g.derived.abil_known[c] & bit != 0 {
            let m = g.derived.abil_off[c] & bit != 0;
            return hit(g, "has_no_ability", m, |g| game_has_no_ability(g, p, card, power_index));
        }
    } else {
        // A new version (or player): start this card's entry afresh.
        g.derived.abil_known[c] = 0;
        g.derived.abil_off[c] = 0;
    }
    let r = game_has_no_ability(g, p, card, power_index);
    if g.st.version == v {
        stored(g);
        if g.derived.abil_v[c] != v || g.derived.abil_p[c] != p as u8 {
            g.derived.abil_known[c] = 0;
            g.derived.abil_off[c] = 0;
        }
        g.derived.abil_v[c] = v;
        g.derived.abil_p[c] = p as u8;
        g.derived.abil_known[c] |= bit;
        if r {
            g.derived.abil_off[c] |= bit;
        }
    }
    r
}

/// The Special Conditions the Pokémon in `target` keeps when it evolves (APR A-05: it recovers from all of them, unless a
/// card says otherwise). No declaration in the pool keeps one (batch 8 C3c: the CheckSpecialConditionRemoval effect had
/// no handler); a card that does needs a modifier read here.
pub fn preserved_conditions(g: &mut Game, p: usize, target: SlotRef) -> R<SVec<u8, 5>> {
    let _ = (g, p, target);
    Ok(SVec::new())
}

/// The lock that forbids an event, if any (`passive::event_locked`): the one query the event's routine asks before the
/// event and legality asks for the event a play would produce.
#[inline]
pub fn event_locked<F: crate::derived::Facts>(g: &mut F, v: &crate::spec::event::EventView) -> R<Option<&'static str>> {
    crate::spec::passive::event_locked(g, v)
}

/// Is the event prevented by a `Prevent` declaration in play (`passive::event_prevented`) or by one an attack left on
/// the Pokémon (`passive::lasting_prevented`: "during your opponent's next turn, prevent all ... done to this Pokémon")?
/// The event routines ask it after the locks.
#[inline]
pub fn event_prevented(g: &mut Game, v: &crate::spec::event::EventView) -> R<bool> {
    if crate::spec::passive::lasting_prevented(g, v)? {
        return Ok(true);
    }
    crate::spec::passive::event_prevented(g, v)
}

// ---------------------------------------------------------------------------
// The facts provider (batch 8 C3d): the game itself (memoized above) or legality's per-decision context (`legal::Ctx`, a
// cache over a `&Game`: no scratch copy of the game). The computations and every reader they reach (the passives'
// cores, the lock pass, `slot_pred_m` / `cond_m` / `num_m`, the event predicates, the evolution rules, the card
// checks) are generic over it.

/// A provider of the derived facts of one game.
pub trait Facts {
    /// The game (plain reads).
    fn g(&self) -> &Game;
    /// Run `f` with no copied attack's source code delegated (a declaration is the card's own, never a delegate: the
    /// card handlers ran so, `Game::call_card`).
    fn undelegated<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T
    where
        Self: Sized;
    fn hp(&mut self, p: usize, s: crate::state::SlotId) -> R<i32>;
    fn types(&mut self, target: SlotRef) -> R<SVec<CardType, 4>>;
    fn provided_energy(&mut self, p: usize, source: SlotRef) -> R<EnergyMap>;
    fn retreat_cost(&mut self, p: usize) -> R<Cost>;
    fn attacks(&mut self, p: usize) -> R<crate::engine::turn::CheckedAttacks>;
    fn attack_cost(&mut self, p: usize, attack: AttackRef) -> R<Cost>;
    fn powers(&mut self, p: usize, card: CardId) -> R<SVec<PowerRef, 8>>;
    fn has_no_ability(&mut self, p: usize, card: CardId, power_index: Option<u8>) -> bool;
    fn tool_off(&mut self, p: usize, card: CardId) -> bool;
}

impl Facts for Game {
    #[inline]
    fn g(&self) -> &Game {
        self
    }
    #[inline]
    fn undelegated<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = self.deleg.take();
        let r = f(self);
        self.deleg = saved;
        r
    }
    #[inline]
    fn hp(&mut self, p: usize, s: crate::state::SlotId) -> R<i32> {
        hp(self, p, s)
    }
    #[inline]
    fn types(&mut self, target: SlotRef) -> R<SVec<CardType, 4>> {
        types(self, target)
    }
    #[inline]
    fn provided_energy(&mut self, p: usize, source: SlotRef) -> R<EnergyMap> {
        provided_energy(self, p, source)
    }
    #[inline]
    fn retreat_cost(&mut self, p: usize) -> R<Cost> {
        retreat_cost(self, p)
    }
    #[inline]
    fn attacks(&mut self, p: usize) -> R<crate::engine::turn::CheckedAttacks> {
        attacks(self, p)
    }
    #[inline]
    fn attack_cost(&mut self, p: usize, attack: AttackRef) -> R<Cost> {
        attack_cost(self, p, attack)
    }
    #[inline]
    fn powers(&mut self, p: usize, card: CardId) -> R<SVec<PowerRef, 8>> {
        powers(self, p, card)
    }
    #[inline]
    fn has_no_ability(&mut self, p: usize, card: CardId, power_index: Option<u8>) -> bool {
        has_no_ability(self, p, card, power_index)
    }
    #[inline]
    fn tool_off(&mut self, p: usize, card: CardId) -> bool {
        tool_off(self, p, card)
    }
}

// ---------------------------------------------------------------------------
// The game's computations: the generic ones below, with what only the game does (the HP read leaves its bonus on the
// spot; `PTCG_VERIFY_CACHE=1` checks each against its check effect, `slow_*`).

/// `PTCG_VERIFY_CACHE=1`: the direct computation against the check effect (when the slow path reads without an error).
#[inline]
fn against_slow<T: PartialEq + std::fmt::Debug>(g: &mut Game, what: &str, r: &T, slow: impl FnOnce(&mut Game) -> R<T>) {
    if crate::game::verify_cache() {
        if let Ok(s) = slow(g) {
            assert_eq!(*r, s, "derived::{what}: the direct computation differs from the check effect");
        }
    }
}

fn game_hp(g: &mut Game, p: usize, s: crate::state::SlotId) -> R<i32> {
    let r = compute_hp(g, p, s)?;
    if let Some(card) = g.st.slot_pokemon(p, s) {
        // The last HP read's bonus stays on the spot (the observation and the canonical JSON read it; batch 8 C7
        // removes the field).
        g.st.players[p].slots[s as usize].hp_bonus = r - g.st.cdef(card).hp;
    }
    against_slow(g, "hp", &r, |g| slow_hp(g, p, s));
    Ok(r)
}

fn game_provided_energy(g: &mut Game, p: usize, source: SlotRef) -> R<EnergyMap> {
    let r = compute_provided_energy(g, p, source)?;
    against_slow(g, "provided_energy", &r, |g| slow_provided_energy(g, p, source));
    Ok(r)
}

fn game_types(g: &mut Game, target: SlotRef) -> R<SVec<CardType, 4>> {
    let r = compute_types(g, target)?;
    against_slow(g, "types", &r, |g| slow_types(g, target));
    Ok(r)
}

fn game_weakness_resistance(g: &mut Game, target: SlotRef) -> R<(SVec<WeaknessV, 4>, SVec<ResistanceV, 4>)> {
    let r = compute_weakness_resistance(g, target)?;
    against_slow(g, "weakness_resistance", &r, |g| slow_weakness_resistance(g, target));
    Ok(r)
}

fn game_retreat_cost(g: &mut Game, p: usize) -> R<Cost> {
    let r = compute_retreat_cost(g, p)?;
    against_slow(g, "retreat_cost", &r, |g| slow_retreat_cost(g, p));
    Ok(r)
}

fn game_attacks(g: &mut Game, p: usize) -> R<crate::engine::turn::CheckedAttacks> {
    let r = compute_attacks(g, p)?;
    against_slow(g, "attacks", &r, |g| slow_attacks(g, p));
    Ok(r)
}

fn game_attack_cost(g: &mut Game, p: usize, attack: AttackRef) -> R<Cost> {
    let r = compute_attack_cost(g, p, attack)?;
    against_slow(g, "attack_cost", &r, |g| slow_attack_cost(g, p, attack));
    Ok(r)
}

fn game_powers(g: &mut Game, p: usize, card: CardId) -> R<SVec<PowerRef, 8>> {
    let r = compute_powers(g, p, card)?;
    against_slow(g, "powers", &r, |g| slow_powers(g, p, card));
    Ok(r)
}

fn game_has_no_ability(g: &mut Game, p: usize, card: CardId, power_index: Option<u8>) -> bool {
    let r = compute_has_no_ability(g, p, card, power_index);
    if crate::game::verify_cache() {
        assert_eq!(r, slow_has_no_ability(g, p, card, power_index), "derived::has_no_ability: the lock pass differs from the Power probe (card {card}, power {power_index:?})");
    }
    r
}

// ---------------------------------------------------------------------------
// The computations (batch 8 C3a-C3d): each fact by direct code over the declarations of its modifier kind that are in
// place (`decl::in_place`: the family's cards, in the propagation order), with today's post-processing
// (`check::finish_attack_cost`, `finish_retreat_cost`, `complete_energy_map`), over any facts provider.

/// The family's cards in place, in the propagation order of the check effects (rank class 0); none without a declarer.
#[inline]
fn declarers(g: &Game, f: usize) -> SVec<CardId, 120> {
    if g.decls.sets[f] == 0 {
        return SVec::new();
    }
    crate::decl::in_place(g, f, 0)
}

/// The HP of the Pokémon in slot `s` of player `p`: its printed HP and each in-place `HpBonus` / `HpMod` that applies.
pub fn compute_hp<F: Facts>(g: &mut F, p: usize, s: crate::state::SlotId) -> R<i32> {
    use crate::spec::passive::Modifier;
    let Some(card) = g.g().st.slot_pokemon(p, s) else { return Ok(0) };
    let target = SlotRef::new(p, s);
    let bonus = g.undelegated(|g| -> R<i32> {
        let mut bonus = 0;
        let ds = declarers(g.g(), crate::decl::fam::HP);
        for me in ds.iter().copied() {
            for ps in crate::decl::passives(g.g(), me) {
                bonus += match &ps.modifier {
                    Modifier::HpBonus(n) => crate::spec::passive::hp_mod_delta(g, me, ps.origin, *n, &crate::spec::SlotPred::Holder, &crate::spec::Cond::True, target)?,
                    Modifier::HpMod(h) => crate::spec::passive::hp_mod_delta(g, me, ps.origin, h.amount, &h.subject, &h.guard, target)?,
                    _ => 0,
                };
            }
        }
        Ok(bonus)
    })?;
    Ok(g.g().st.cdef(card).hp + bonus)
}

/// The Energy map of the Pokémon in `source`, asked by `p`: the in-place `ProvidesEnergy` / `ProvidesEnergyBoost`
/// entries in the propagation order, then the plain Energy attached (`check::complete_energy_map`).
pub fn compute_provided_energy<F: Facts>(g: &mut F, p: usize, source: SlotRef) -> R<EnergyMap> {
    use crate::spec::passive::Modifier;
    let mut map: EnergyMap = SVec::new();
    g.undelegated(|g| -> R {
        let ds = declarers(g.g(), crate::decl::fam::PROVIDES);
        for me in ds.iter().copied() {
            for ps in crate::decl::passives(g.g(), me) {
                match &ps.modifier {
                    Modifier::ProvidesEnergy(pe) => crate::spec::passive::provides_energy_into(g, me, pe, p as u8, source, &mut map)?,
                    Modifier::ProvidesEnergyBoost(b) => crate::spec::passive::provides_energy_boost_into(g, me, ps.origin, b, p, source, &mut map)?,
                    _ => {}
                }
            }
        }
        Ok(())
    })?;
    crate::engine::check::complete_energy_map(g.g(), source, &mut map);
    Ok(map)
}

/// The types of the Pokémon in `target`: its printed types, replaced by each in-place `TypeOverride` that applies (the
/// last one wins).
pub fn compute_types<F: Facts>(g: &mut F, target: SlotRef) -> R<SVec<CardType, 4>> {
    use crate::spec::passive::Modifier;
    let mut types = crate::engine::game_effect::pokemon_types(g.g(), target);
    g.undelegated(|g| -> R {
        let ds = declarers(g.g(), crate::decl::fam::TYPE);
        for me in ds.iter().copied() {
            for ps in crate::decl::passives(g.g(), me) {
                if let Modifier::TypeOverride(t) = &ps.modifier {
                    if let Some(set) = crate::spec::passive::type_override(g, me, ps.origin, t, target)? {
                        types.clear();
                        for ty in set.iter() {
                            types.push(*ty);
                        }
                    }
                }
            }
        }
        Ok(())
    })?;
    Ok(types)
}

/// The Weakness and Resistance of the Pokémon in `target`: the printed ones, each in-place `WeaknessOverride`, then a
/// lasting "no Weakness" on the Pokémon.
pub fn compute_weakness_resistance<F: Facts>(g: &mut F, target: SlotRef) -> R<(SVec<WeaknessV, 4>, SVec<ResistanceV, 4>)> {
    use crate::spec::passive::Modifier;
    let (mut weakness, resistance) = match crate::engine::game_effect::stats_effect(g.g(), target) {
        Effect::CheckPokemonStats { weakness, resistance, .. } => (weakness, resistance),
        _ => (SVec::new(), SVec::new()),
    };
    g.undelegated(|g| -> R {
        let ds = declarers(g.g(), crate::decl::fam::STATS);
        for me in ds.iter().copied() {
            for ps in crate::decl::passives(g.g(), me) {
                if let Modifier::WeaknessOverride(w) = &ps.modifier {
                    if let Some(t) = crate::spec::passive::weakness_override(g, me, ps.origin, w, target)? {
                        weakness.clear();
                        weakness.push(WeaknessV { card_type: t, value: None });
                    }
                }
            }
        }
        Ok(())
    })?;
    // "No Weakness during your opponent's next turn" (a lasting effect on the Pokémon), after the declarations.
    if g.g().st.slot(target.p as usize, target.s).no_weakness_next_turn {
        weakness.clear();
    }
    Ok((weakness, resistance))
}

/// Player `p`'s Active Pokémon's Retreat Cost: the printed one with the lasting increase, each in-place `RetreatCost`,
/// then the last steps (`check::finish_retreat_cost`).
pub fn compute_retreat_cost<F: Facts>(g: &mut F, p: usize) -> R<Cost> {
    use crate::spec::passive::Modifier;
    let mut acc = crate::engine::check::RetreatCostAcc { p: p as u8, cost: crate::engine::retreat::check_retreat_cost_base(g.g(), p), no_cost: false, reduction: 0 };
    g.undelegated(|g| -> R {
        let ds = declarers(g.g(), crate::decl::fam::RETREAT_COST);
        for me in ds.iter().copied() {
            for ps in crate::decl::passives(g.g(), me) {
                if let Modifier::RetreatCost(c) = &ps.modifier {
                    crate::spec::passive::retreat_cost_into(g, me, ps.origin, c, &mut acc)?;
                }
            }
        }
        Ok(())
    })?;
    Ok(crate::engine::check::finish_retreat_cost(acc))
}

/// The attacks player `p`'s Active Pokémon may use: its Tool's attacks, then each in-place `GrantAttacks` /
/// `BenchAttacks`.
pub fn compute_attacks<F: Facts>(g: &mut F, p: usize) -> R<crate::engine::turn::CheckedAttacks> {
    use crate::spec::passive::Modifier;
    let mut acc: crate::engine::turn::CheckedAttacks = match crate::engine::turn::check_attacks_effect(g.g(), p) {
        Effect::CheckPokemonAttacks { attacks, copied, .. } => (attacks, copied),
        _ => (SVec::new(), SVec::new()),
    };
    g.undelegated(|g| -> R {
        let ds = declarers(g.g(), crate::decl::fam::GRANT_ATTACKS);
        for me in ds.iter().copied() {
            for ps in crate::decl::passives(g.g(), me) {
                match &ps.modifier {
                    Modifier::GrantAttacks(_) => crate::spec::passive::grant_attacks_into(g, me, ps.origin, p, &mut acc)?,
                    Modifier::BenchAttacks(_) => crate::spec::passive::bench_attacks_into(g, me, ps.origin, p, &mut acc)?,
                    _ => {}
                }
            }
        }
        Ok(())
    })?;
    Ok(acc)
}

/// The current cost of `attack` for player `p`: the printed cost, each in-place `AttackCost`, then the last steps
/// (`check::finish_attack_cost`).
pub fn compute_attack_cost<F: Facts>(g: &mut F, p: usize, attack: AttackRef) -> R<Cost> {
    use crate::spec::passive::Modifier;
    let mut cost: Cost = SVec::new();
    for &c in crate::engine::attack::attack_def(g.g(), attack).cost {
        cost.push(c);
    }
    let mut acc = crate::engine::check::AttackCostAcc { p: p as u8, attack, cost, set_cost: None, ignore_colorless: false, reduction: 0, any_reduction: false };
    g.undelegated(|g| -> R {
        let ds = declarers(g.g(), crate::decl::fam::ATTACK_COST);
        for me in ds.iter().copied() {
            for ps in crate::decl::passives(g.g(), me) {
                if let Modifier::AttackCost(c) = &ps.modifier {
                    crate::spec::passive::attack_cost_into(g, me, ps.origin, c, &mut acc)?;
                }
            }
        }
        Ok(())
    })?;
    crate::engine::check::finish_attack_cost(g, acc)
}

/// The powers the card has: the Ability locks' removals, by direct code (`passive::powers_now`).
pub fn compute_powers<F: Facts>(g: &mut F, p: usize, card: CardId) -> R<SVec<PowerRef, 8>> {
    crate::spec::passive::powers_now(g, p, card)
}

/// "Has no Ability": the lock pass for the generic probe or the power (`passive::power_blocked`).
pub fn compute_has_no_ability<F: Facts>(g: &mut F, p: usize, card: CardId, power_index: Option<u8>) -> bool {
    let power = PowerRef { card, index: power_index.unwrap_or(crate::prefabs::PROBE_GENERIC) };
    crate::spec::passive::power_blocked(g, p, power, card, true)
}

/// Is the text of the Tool `card` (player `p`'s) off: "Pokémon Tools have no effect" for some turns (an attack's lasting
/// effect), or a declaration that turns Tools off in place (Jamming Tower: `PreventWhat::ToolEffects`, unless its own
/// Stadium effect is off on the Pokémon holding the Tool)? (Batch 8 C3b; the Tool probe was `prefabs::is_tool_blocked`.)
pub fn tool_off(g: &mut Game, p: usize, card: CardId) -> bool {
    let r = compute_tool_off(g, p, card);
    if crate::game::verify_cache() {
        assert_eq!(r, slow_tool_off(g, p, card), "derived::tool_off differs from the Tool probe (card {card})");
    }
    r
}

/// [`tool_off`] by direct code (plain reads).
pub fn compute_tool_off(g: &Game, p: usize, card: CardId) -> bool {
    use crate::spec::passive::{Modifier, PreventWhat};
    if stadium_off(g) {
        return true;
    }
    if g.decls.sets[crate::decl::fam::TOOLS_OFF] == 0 {
        return false;
    }
    for me in crate::decl::in_place(g, crate::decl::fam::TOOLS_OFF, 0).iter().copied() {
        for ps in crate::decl::passives(g, me) {
            let Modifier::Prevent(pv) = &ps.modifier else { continue };
            if pv.what != PreventWhat::ToolEffects || crate::spec::passive::locate(g, me, ps.origin).is_none() {
                continue;
            }
            // The Pokémon of player `p` holding the Tool: the Stadium's own effect off there spares it.
            let held = crate::prefabs::for_each_pokemon(g, p, crate::types::PlayerType::BottomPlayer).iter().any(|(s, _, _)| g.st.slot(p, *s).tools.contains(card));
            if held && stadium_off(g) {
                continue;
            }
            return true;
        }
    }
    false
}

/// Is a Stadium's text off on the Pokémon it would affect: "Stadium and Pokémon Tool cards have no effect" for some turns
/// (no declaration in the pool turns a Stadium's effect off on a Pokémon; batch 8 C3b, the Stadium probe had no
/// handler).
#[inline]
pub fn stadium_off(g: &Game) -> bool {
    g.st.players.iter().any(|pl| pl.stadium_and_tool_have_no_effect_turns_remaining > 0)
}

/// Is the text of the attached Special Energy `card` off? No declaration in the pool turns it off (batch 8 C3b: the
/// SpecialEnergy and Energy probes had no handler); the unit tests' stand-in for one (Temple of Sinnoh, id2033) says so.
#[inline]
pub fn special_energy_off(g: &Game, card: CardId) -> bool {
    let _ = (g, card);
    #[cfg(test)]
    if crate::prefabs::TEST_SPECIAL_ENERGY_OFF.with(|b| b.get()) {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// The slow path: the check effects dispatched to the cards (kept until the batch 8 closeout as the reference).

/// `CheckHp` (`check::check_hp`).
pub fn slow_hp(g: &mut Game, p: usize, s: crate::state::SlotId) -> R<i32> {
    crate::engine::check::check_hp(g, p, s)
}

/// `CheckProvidedEnergy`.
pub fn slow_provided_energy(g: &mut Game, p: usize, source: SlotRef) -> R<EnergyMap> {
    crate::engine::attack::provided_energy_read(g, p, source)
}

/// `CheckPokemonType`, from the printed types.
pub fn slow_types(g: &mut Game, target: SlotRef) -> R<SVec<CardType, 4>> {
    let seed = crate::engine::game_effect::pokemon_types(g, target);
    let (e, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: seed })?;
    Ok(match e {
        Effect::CheckPokemonType { card_types, .. } => card_types,
        _ => SVec::new(),
    })
}

/// `CheckPokemonStats`.
pub fn slow_weakness_resistance(g: &mut Game, target: SlotRef) -> R<(SVec<WeaknessV, 4>, SVec<ResistanceV, 4>)> {
    let (e, _) = g.run_fx(crate::engine::game_effect::stats_effect(g, target))?;
    Ok(match e {
        Effect::CheckPokemonStats { weakness, resistance, .. } => (weakness, resistance),
        _ => (SVec::new(), SVec::new()),
    })
}

/// `CheckRetreatCost`.
pub fn slow_retreat_cost(g: &mut Game, p: usize) -> R<Cost> {
    crate::engine::retreat::retreat_cost_read(g, p)
}

/// `CheckPokemonAttacks`.
pub fn slow_attacks(g: &mut Game, p: usize) -> R<crate::engine::turn::CheckedAttacks> {
    crate::engine::turn::read_attack_list(g, p)
}

/// `CheckAttackCost`.
pub fn slow_attack_cost(g: &mut Game, p: usize, attack: AttackRef) -> R<Cost> {
    crate::engine::attack::attack_cost_read(g, p, attack)
}

/// `CheckPokemonPowers`, from the printed powers.
pub fn slow_powers(g: &mut Game, p: usize, card: CardId) -> R<SVec<PowerRef, 8>> {
    let mut powers = SVec::new();
    for i in 0..g.st.cdef(card).powers.len() {
        powers.push(PowerRef { card, index: i as u8 });
    }
    let (e, _) = g.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: card, powers })?;
    Ok(match e {
        Effect::CheckPokemonPowers { powers, .. } => powers,
        _ => SVec::new(),
    })
}

/// The `Power` probe (`prefabs::is_ability_blocked`).
pub fn slow_has_no_ability(g: &mut Game, p: usize, card: CardId, power_index: Option<u8>) -> bool {
    crate::prefabs::is_ability_blocked(g, p, card, power_index)
}

/// The Tool probe (`prefabs::tool_probe_blocked`).
pub fn slow_tool_off(g: &mut Game, p: usize, card: CardId) -> bool {
    crate::prefabs::tool_probe_blocked(g, p, card)
}

