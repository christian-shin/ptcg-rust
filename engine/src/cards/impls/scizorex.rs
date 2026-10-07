//! Scizor ex (TEF 111): Steel Wing — 70; during your opponent's next turn
//! this Pokémon takes 50 less damage from attacks. Cross Breaker — 120x;
//! discard up to 2 [M] Energy from this Pokémon, 120 damage for each card
//! discarded.
//!
//! Twinleaf: Steel Wing sets `player.active.damageReductionNextTurn = 50`.
//! Cross Breaker opens a DiscardEnergyPrompt on the Active (Energy cards, the
//! ones that don't provide [M] blocked, min 0, max 2, no cancel) even when nothing matches; an empty
//! answer sets the damage to 0, otherwise each transfer is a MOVE_CARDS to
//! the discard (no DiscardCardsEffect) and the damage is 120 x transfers.
//! R7A (ruling 1874): the Energy is chosen first, the damage is done, then the Energy is discarded (`move_cards_after_damage`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Scizorex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].damage_reduction_next_turn = 50;
        }
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        // "[M] Energy": every Energy that provides [M], including one that provides every type (Advanced
        // Rulebook D-08); the others are blocked in the prompt.
        let a = g.st.players[p].active;
        let filter = Filter::super_type(SuperType::Energy);
        let mut o = MoveOpts { allow_cancel: false, min: 0, max: Some(2), ..Default::default() };
        if let Some(b) = blocked_non_type_energy(g, p, a, ct::METAL)? {
            o.blocked_map.push((CardTarget::new(PlayerType::BottomPlayer, SlotType::Active, 0), b));
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_ENERGIES_TO_DISCARD",
            PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o },
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
    let e = f.e[0];
    let r = (|| -> R {
        let transfers = match results.first().copied() {
            Some(Res::CardsFrom(t)) => t,
            _ => SVec::new(),
        };
        if transfers.is_empty() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = 0;
            }
            return Ok(());
        }
        let n = transfers.len() as i32;
        for (from, c) in transfers.iter().copied() {
            let source = get_target(&g.st, p, from)?;
            move_cards_after_damage(g, e, source.list(), ListRef::Discard(p as u8), &[c], me)?;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = n * 120;
            }
        }
        Ok(())
    })();
    g.release_fx(e);
    r
}
