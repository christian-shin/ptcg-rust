//! Zacian ex (SVP 198): Steel Armament — 20; search your deck for a Basic
//! [M] Energy card and attach it to this Pokémon, then shuffle. Slashing
//! Strike — 210; during your next turn this Pokémon can't use Slashing Strike.
//!
//! Twinleaf: ATTACH_UP_TO_X_ENERGY_FROM_DECK_TO_Y_OF_YOUR_POKEMON(1, 1, {
//! destinationSlots: [ACTIVE], energyFilter: { energyType: BASIC, name:
//! 'Metal Energy' } }): no prompt on an empty deck; a non-cancellable
//! AttachEnergyPrompt on the deck (min 0, max 1); every transfer is an
//! AttachEnergyEffect, whose reducer only moves cards out of the HAND. A card
//! chosen from the deck therefore stays in the deck while being listed in the
//! slot's `energies` (Twinleaf bug kept). Then SHUFFLE_DECK.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ZacianexSVPPool", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
        o.allow_cancel = false;
        o.min = 0;
        o.max = 1;
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Metal Energy"),
            ..Filter::none()
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_CARDS",
            PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            let pending = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
            if !pending.iter().any(|n| *n == "Slashing Strike") {
                pending.push("Slashing Strike");
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
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
        g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
    }
    shuffle_deck(g, p);
    Ok(())
}
