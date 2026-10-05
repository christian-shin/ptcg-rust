//! Flutter Mane (TEF): Midnight Fluttering — as long as this Pokémon is in the
//! Active Spot, your opponent's Active Pokémon has no Abilities, except for
//! Midnight Fluttering. Hex Hurl — 90; put 2 damage counters on your
//! opponent's Benched Pokémon in any way you like.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, exemptPowerNames
//! ['Midnight Fluttering'], hand/discard powers locked too, error
//! BLOCKED_BY_ABILITY): strips Abilities from CheckPokemonPowersEffect and
//! throws on PowerEffect when the callback says so: this card's list (findCardList
//! throws INVALID_GAME_STATE when it is in none) must have this card as its
//! owner's Active, the checked card's list must be the opponent's Active, and
//! LOCKER_ABILITY_APPLIES (activation-order check against other ability
//! lockers, then a real PowerEffect for Midnight Fluttering by the owner that
//! must not throw). The name exemption doesn't apply to a lock probe (its
//! power is named 'test'). Hex Hurl is PUT_X_DAMAGE_COUNTERS_IN_ANY_WAY_YOU_LIKE
//! (2, Bench).
//!
//! Fixed (phase 4b, R2): the lock also stripped Hide 'n' Sneak (Shuppet,
//! Banette, ...) from the Active Pokémon; Hide 'n' Sneak takes precedence over
//! Midnight Fluttering (ruling 1877), so such a card is skipped.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "FlutterMane", mask: mask(&[k::CHECK_POKEMON_POWERS, k::POWER, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

/// `IS_POWER_SUBJECT_TO_ABILITY_LOCK` with this card's options.
fn subject(g: &Game, power: PowerRef, probe: bool) -> bool {
    if power.index == PROBE_GENERIC {
        return true;
    }
    let d = &g.st.cdef(power.card).powers[power.index as usize];
    if d.power_type != PowerType::Ability as u8 || d.exempt_from_ability_lock {
        return false;
    }
    probe || d.name != "Midnight Fluttering"
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
fn is_locked(g: &mut Game, me: CardId, card: CardId) -> R<bool> {
    let my_list = match g.st.locate(me) {
        Some(l) => l,
        None => bail!("INVALID_GAME_STATE"),
    };
    let owner = match my_list.owner() {
        Some(o) => o,
        None => bail!("INVALID_GAME_STATE"),
    };
    if g.st.active_pokemon(owner) != Some(me) {
        return Ok(false);
    }
    let opponent = 1 - owner;
    let target_list = match g.st.locate(card) {
        Some(l) => l,
        None => bail!("INVALID_GAME_STATE"),
    };
    if target_list != ListRef::Slot(opponent as u8, g.st.players[opponent].active) {
        return Ok(false);
    }
    // Hide 'n' Sneak takes precedence over Midnight Fluttering (it keeps working).
    if g.st.cdef(card).powers.iter().any(|pw| pw.name == "Hide 'n' Sneak") {
        return Ok(false);
    }
    // LOCKER_ABILITY_APPLIES
    let lock_card = g.st.cdef(card).powers.iter().any(|pw| pw.ability_lock);
    if lock_card && !can_suppress(g, me, card) {
        return Ok(false);
    }
    let own = PowerRef { card: me, index: 0 };
    Ok(g.run_fx(Effect::Power { p: owner as u8, power: own, card: me, target: None, probe: false }).is_ok())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckPokemonPowers { target, powers, .. } => {
            if is_locked(g, me, target)? {
                let mut out = SVec::new();
                for pw in powers.iter() {
                    if !subject(g, *pw, false) {
                        out.push(*pw);
                    }
                }
                if let Effect::CheckPokemonPowers { powers, .. } = g.e_mut(e) {
                    *powers = out;
                }
            }
        }
        Effect::Power { power, card, probe, .. } => {
            if subject(g, power, probe) && is_locked(g, me, card)? {
                bail!("BLOCKED_BY_ABILITY");
            }
        }
        _ => {}
    }

    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let opl = &g.st.players[o];
        let has_benched = opl.bench.iter().any(|b| !opl.slots[*b as usize].cards.is_empty());
        if !has_benched {
            return Ok(());
        }
        let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
        for t in slot_targets(&g.st, p, PlayerType::TopPlayer, &[SlotType::Active as u8, SlotType::Bench as u8]) {
            max_allowed.push((t, 9999));
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::PutDamage {
                player_type: PlayerType::TopPlayer,
                slots,
                damage: 20,
                max_allowed,
                allow_cancel: false,
                blocked: SVec::new(),
                allow_partial: false,
                damage_multiple: 10,
            },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| -> R {
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let map: SVec<(CardTarget, i32), 16> = match results.first().copied().unwrap_or(Res::Null) {
            Res::DamageMap(m) => m,
            _ => SVec::new(),
        };
        for (t, damage) in map.iter() {
            let target = get_target(&g.st, p as usize, *t)?;
            let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::PutCounters { b, damage: *damage })?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
