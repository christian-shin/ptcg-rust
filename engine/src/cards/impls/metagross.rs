//! Metagross (M4 / CRI): Bounce Back — 60, after attacking switch out your
//! opponent's Active Pokémon (your opponent chooses the new Active Pokémon;
//! fixed in phase 4b: Twinleaf let the attacker choose and switched without
//! dispatching the switch effects). Metallic Hammer — 150+, you may discard 3
//! [M] Energy from this Pokémon for 150 more damage.
//!
//! Fixed (phase 4b, R7F-9): Bounce Back switched the Defending Pokémon with a
//! plain prompt, so Mist Energy (or any effect that prevents the effects of
//! attacks) did not stop it (ruling 1574); it now goes through
//! SWITCH_OUT_OPPONENT_ACTIVE_POKEMON like Bayleef M1S (a preventable
//! SwitchOutOpponentsActiveEffect before the opponent's prompt, another with
//! the chosen Bench Pokémon for the switch). Metallic Hammer's choice is now
//! always offered: a copy of the attack by a Pokémon with fewer than 3 [M]
//! Energy (Slowking SCR's Seek Inspiration) may still do the 150 more damage
//! and discards as many [M] Energy as it can (ruling 1822).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Metagross@Metagross M4",
    mask: mask(&[k::ATTACK, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

/// Reduce a SwitchOutOpponentsActiveEffect built on `new AttackEffect(player,
/// opponent, attack)`; returns `preventDefault`.
fn run_switch_out(g: &mut Game, p: usize, o: usize, attack: AttackRef, bench_target: Option<SlotRef>) -> R<bool> {
    let source = SlotRef::new(p, g.st.players[p].active);
    let atk = g.new_fx(Effect::Attack {
        p: p as u8,
        opp: o as u8,
        attack,
        damage: 0,
        ignore_weakness: false,
        ignore_resistance: false,
        ignore_defender_effects: false,
        source,
        barrage_used: false,
    });
    let target = SlotRef::new(o, g.st.players[o].active);
    let b = AtkBase { attack_effect: atk, player: p as u8, opponent: o as u8, attack, source, target };
    let r = g.run_fx(Effect::SwitchOutOpponentsActive { b, bench_target });
    g.release_fx(atk);
    Ok(r?.1)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let (p, attack) = match *g.e(e) {
            Effect::AfterAttack { p, attack, .. } => (p as usize, attack),
            _ => return Ok(()),
        };
        let o = 1 - p;
        let has_bench = g.st.players[o].bench.iter().any(|s| !g.st.slot(o, *s).cards.is_empty());
        if !has_bench {
            return Ok(());
        }
        if run_switch_out(g, p, o, attack, None)? {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(3);
        f.a[0] = p as i32;
        f.a[1] = attack.index as i32;
        f.a[2] = attack.card as i32;
        let id = g.player_id(o);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_SWITCH",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        confirmation_prompt(g, p, "WANT_TO_DISCARD_ENERGY", Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        3 => {
            let attack = AttackRef { card: f.a[2] as CardId, index: f.a[1] as u8 };
            let sel = first.slots();
            if sel.is_empty() {
                return Ok(());
            }
            run_switch_out(g, p, 1 - p, attack, Some(sel[0]))?;
            Ok(())
        }
        1 => {
            if !first.as_bool() {
                g.release_fx(atk);
                return Ok(());
            }
            if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                *damage += 150;
            }
            // DISCARD_UP_TO_X_TYPE_ENERGY_FROM_YOUR_POKEMON(effect, 3, M, 3, [ACTIVE]).
            let mut available = 0;
            let mut o = MoveOpts { allow_cancel: false, ..Default::default() };
            for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if t.slot != SlotType::Active {
                    continue;
                }
                // ENERGY_CARDS_THAT_PROVIDE_TYPE: Energy as it is provided now (Legacy Energy provides
                // every type: Advanced Rulebook D-08).
                let providing = energy_cards_that_provide_type(g, p, s, ct::METAL)?;
                available += providing.len();
                let mut b = Blocked::default();
                for (i, c) in g.st.slot(p, s).cards.iter().enumerate() {
                    if !providing.contains(&c) {
                        b.push(i as u8);
                    }
                }
                o.blocked_map.push((t, b));
            }
            if available == 0 {
                g.release_fx(atk);
                return Ok(());
            }
            let max = 3.min(available) as u8;
            o.min = 3.min(max);
            o.max = Some(max);
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.e[0] = atk;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_ENERGIES_TO_DISCARD",
                PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
                Cont::Card { card: _me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let r = discard_chosen(g, p, atk, first);
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}

fn discard_chosen(g: &mut Game, p: usize, atk: EffId, first: Res) -> R {
    let transfers = match first {
        Res::CardsFrom(t) => t,
        _ => return Ok(()),
    };
    if transfers.is_empty() {
        return Ok(());
    }
    let providing = energy_cards_that_provide_type(g, p, g.st.players[p].active, ct::METAL)?;
    if !transfers.iter().all(|(_, c)| providing.contains(c)) {
        bail!("INVALID_PROMPT_RESULT");
    }
    // discardTransfersAsEffects: one DiscardCardsEffect per source, in first-seen order.
    let mut groups: Vec<(SlotRef, SVec<CardId, 64>)> = Vec::new();
    for (from, c) in transfers.iter().copied() {
        let s = get_target(&g.st, p, from)?;
        match groups.iter_mut().find(|(x, _)| *x == s) {
            Some((_, v)) => v.push(c),
            None => {
                let mut v = SVec::new();
                v.push(c);
                groups.push((s, v));
            }
        }
    }
    let (pp, opp, attack, source) = match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    for (s, cards) in groups {
        let b = AtkBase { attack_effect: atk, player: pp, opponent: opp, attack, source, target: s };
        g.run_fx(Effect::DiscardCards { b, cards })?;
    }
    Ok(())
}
