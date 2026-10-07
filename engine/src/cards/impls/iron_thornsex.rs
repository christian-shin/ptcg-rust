//! Iron Thorns ex (TWM / PRE): Initialization — as long as this Pokémon is in
//! the Active Spot, Pokémon with a Rule Box in play (both yours and your
//! opponent's) have no Abilities, except for Future Pokémon. Volt Cyclone —
//! 140; move an Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, allowUseFromHand,
//! allowUseFromDiscard, respectExemptFromInitialize, error
//! BLOCKED_BY_ABILITY): strips Abilities from CheckPokemonPowersEffect and
//! throws on PowerEffect when this card is the top card of either Active, the
//! checked card sits on a Pokémon slot or is still in the hand (a Pokémon
//! being benched or evolved: it is in play, so its on-play Ability is locked
//! too; fixed in phase 4b), is not Future and has a Rule Box,
//! and Initialization itself applies (a real PowerEffect for it, by the
//! player whose Active it is, must not throw; on a PowerEffect also an
//! EffectOfAbilityEffect against the target slot must keep its target).
//! Initialization's own EffectOfAbilityEffect drops the target when it is a
//! Future Pokémon. Volt Cyclone prompts after the attack (AfterAttackEffect)
//! for 1 Energy to move to the Bench.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "IronThornsex",
    mask: mask(&[k::CHECK_POKEMON_POWERS, k::POWER, k::EFFECT_OF_ABILITY, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

/// `IS_POWER_SUBJECT_TO_ABILITY_LOCK` with this card's options.
fn subject(g: &Game, power: PowerRef) -> bool {
    if power.index == PROBE_GENERIC {
        return true;
    }
    let d = &g.st.cdef(power.card).powers[power.index as usize];
    d.power_type == PowerType::Ability as u8 && !d.exempt_from_ability_lock && !d.exempt_from_initialize && !d.use_from_hand && !d.use_from_discard
}

fn lock_order(g: &Game, c: CardId) -> i32 {
    match g.st.locate(c) {
        Some(ListRef::Slot(p, s)) => g.st.slot(p as usize, s).ability_lock_activation_order,
        _ => 0,
    }
}

fn is_turn_players_card(g: &Game, c: CardId) -> bool {
    match g.st.locate(c).and_then(|l| l.owner()) {
        Some(o) => g.st.active_player as usize == o,
        None => false,
    }
}

/// `CAN_SUPPRESS_ABILITY_LOCKER(state, suppressor, target)`.
fn can_suppress(g: &Game, suppressor: CardId, target: CardId) -> bool {
    let s = lock_order(g, suppressor);
    let t = lock_order(g, target);
    if s == 0 && t == 0 {
        return is_turn_players_card(g, suppressor);
    }
    if s == 0 {
        return false;
    }
    if t == 0 {
        return true;
    }
    if s < t {
        return true;
    }
    if s > t {
        return false;
    }
    is_turn_players_card(g, suppressor)
}

/// The `HANDLE_ABILITY_LOCK` callback.
fn is_locked(g: &mut Game, me: CardId, player: usize, card: CardId, power_effect: bool) -> R<bool> {
    if g.st.active_pokemon(player) != Some(me) && g.st.active_pokemon(1 - player) != Some(me) {
        return Ok(false);
    }
    // A Pokémon still in the hand is being played (benched / evolved), so it is in play: locked too.
    let slot = match g.st.locate(card) {
        Some(ListRef::Slot(q, s)) => Some(SlotRef::new(q as usize, s)),
        Some(ListRef::Hand(_)) => None,
        _ => return Ok(false),
    };
    let d = g.st.cdef(card);
    if d.has_tag(tag::FUTURE) || !d.has_rule_box() {
        return Ok(false);
    }
    let locker_owner = if g.st.active_pokemon(player) == Some(me) { player } else { 1 - player };
    // LOCKER_ABILITY_APPLIES
    let lock_card = g.st.cdef(card).powers.iter().any(|pw| pw.ability_lock);
    if lock_card && !can_suppress(g, me, card) {
        return Ok(false);
    }
    let own = PowerRef { card: me, index: 0 };
    if g.run_fx(Effect::Power { p: locker_owner as u8, power: own, card: me, target: None, probe: false }).is_err() {
        return Ok(false);
    }
    if power_effect {
        // CAN_APPLY_LOCK_TO_TARGET (a card in the hand is not on a Pokémon slot: true)
        let slot = match slot {
            Some(slot) => slot,
            None => return Ok(true),
        };
        return Ok(match g.run_fx(Effect::EffectOfAbility { p: locker_owner as u8, power: own, card: me, target: Some(slot) }) {
            Ok((Effect::EffectOfAbility { target, .. }, _)) => target.is_some(),
            _ => false,
        });
    }
    Ok(true)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EffectOfAbility { power, card, target, .. } = *g.e(e) {
        if card == me && power == (PowerRef { card: me, index: 0 }) {
            if let Some(t) = target {
                if let Some(c) = g.st.slot_pokemon(t.p as usize, t.s) {
                    if g.st.cdef(c).has_tag(tag::FUTURE) {
                        if let Effect::EffectOfAbility { target, .. } = g.e_mut(e) {
                            *target = None;
                        }
                    }
                }
            }
        }
    }

    match *g.e(e) {
        Effect::CheckPokemonPowers { p, target, powers } => {
            if is_locked(g, me, p as usize, target, false)? {
                let mut out = SVec::new();
                for pw in powers.iter() {
                    if !subject(g, *pw) {
                        out.push(*pw);
                    }
                }
                if let Effect::CheckPokemonPowers { powers, .. } = g.e_mut(e) {
                    *powers = out;
                }
            }
        }
        Effect::Power { p, power, card, .. } => {
            if subject(g, power) && is_locked(g, me, p as usize, card, true)? {
                bail!("BLOCKED_BY_ABILITY");
            }
        }
        _ => {}
    }

    if after_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::AfterAttack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has_bench = g.st.players[p].bench.iter().any(|s| !g.st.players[p].slots[*s as usize].cards.is_empty());
        if !has_bench {
            return Ok(());
        }
        let a = g.st.players[p].active;
        let mut o = AttachOpts::new(g.st.slot(p, a).cards.len() as u8);
        o.allow_cancel = false;
        o.min = 1;
        o.max = 1;
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Slot(p as u8, a), player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        let a = g.st.players[p].active;
        move_cards(g, ListRef::Slot(p as u8, a), target.list(), &[c], me)?;
    }
    Ok(())
}
