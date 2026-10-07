//! Iron Treads (TEF 118): Dual Core — as long as this Pokémon has a Future
//! Booster Energy Capsule attached, it is [F] and [M] type. Wheel Pass — 60;
//! move an Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Twinleaf: Dual Core replaces `cardTypes` with [F, M] on a
//! CheckPokemonTypeEffect whose target's Pokémon is this card, when a tool
//! named "Future Booster Energy Capsule" is attached and the ability isn't
//! blocked. Wheel Pass (after the attack): nothing unless a Benched Pokémon
//! exists, this slot holds an Energy card and is the Active; then a
//! non-cancellable AttachEnergyPrompt (Active's cards → Bench, min 1 max 1)
//! whose callback MOVE_CARDS each transfer from the Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "IronTreadsTEFPool",
    mask: mask(&[k::CHECK_POKEMON_TYPE, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckPokemonType { target, .. } = *g.e(e) {
        let (tp, ts) = (target.p as usize, target.s);
        if g.st.slot_pokemon(tp, ts) != Some(me) {
            return Ok(());
        }
        let has_capsule = g.st.slot(tp, ts).tools.iter().any(|t| g.st.cdef(t).name == "Future Booster Energy Capsule");
        if !has_capsule {
            return Ok(());
        }
        if is_ability_blocked(g, tp, me, None) {
            return Ok(());
        }
        if let Effect::CheckPokemonType { card_types, .. } = g.e_mut(e) {
            card_types.clear();
            card_types.push(ct::FIGHTING);
            card_types.push(ct::METAL);
        }
        return Ok(());
    }

    if after_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::AfterAttack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let has_bench = pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty());
        let source = (0..pl.slots.len() as u8).find(|s| g.st.slot(p, *s).cards.contains(me));
        let source = match source {
            Some(s) => s,
            None => return Ok(()),
        };
        let has_energy = g.st.slot(p, source).cards.iter().any(|c| g.st.cdef(c).is_energy());
        if !has_bench || !has_energy || source != pl.active {
            return Ok(());
        }
        let n = g.st.slot(p, source).cards.len() as u8;
        let mut o = AttachOpts::new(n);
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
            PromptKind::AttachEnergy {
                cards: ListRef::Slot(p as u8, source),
                player_type: PlayerType::BottomPlayer,
                slots,
                filter: Filter::super_type(SuperType::Energy),
                o,
            },
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
        let src = ListRef::Slot(p as u8, g.st.players[p].active);
        move_cards(g, src, target.list(), &[c], me)?;
    }
    Ok(())
}
