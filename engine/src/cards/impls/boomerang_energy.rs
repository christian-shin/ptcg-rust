//! Boomerang Energy (TWM): provides [C]. If discarded by an effect of an
//! attack of the Pokémon it is attached to, attach it from the discard pile
//! to that Pokémon after attacking.
//!
//! Twinleaf quirk kept: the card is re-attached to whatever is Active then.
//!
//! Fixed (phase 4b, R7F-10; ruling 1650): it was re-attached at EndTurn; it is
//! now re-attached in AfterAttackTriggersEffect, once the attack's effects and the
//! Energy choices they ask after the damage are done (EndTurn stays as the fallback
//! for a discard no AfterAttackTriggersEffect followed), before the effects that trigger
//! on the Defending Pokémon (Handheld Fan) resolve: AfterAttackEffect reaches
//! Pokémon, then Energy, then Trainers.
//!
//! Fixed (phase 4b, W4): the re-attach was armed by a marker set on the
//! AttackEffect, but the attacker's own handler runs before its attached
//! Energy sees that effect, so a discard made synchronously by the attack
//! (Volt Strike, "discard all Energy") happened before the marker existed and
//! the card was never re-attached. It is now armed directly by a
//! DiscardCardsEffect from the Active holding this card that lists this card.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "BoomerangEnergy",
    mask: mask(&[k::DISCARD_CARDS, k::END_TURN, k::AFTER_ATTACK_TRIGGERS]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn discarded() -> crate::markers::MarkerName {
    crate::marker!("BOOMERANG_DISCARDED_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::DiscardCards { b, ref cards } = *g.e(e) {
        let pu = b.player as usize;
        if cards.contains(&me) && g.st.slot(b.source.p as usize, b.source.s).cards.contains(me) && g.st.players[pu].active == b.source.s && b.source.p == b.player {
            if is_special_energy_blocked(g, pu, me, b.source, false) {
                return Ok(());
            }
            g.st.players[pu].marker.add(discarded(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        }
    }

    let p = match *g.e(e) {
        Effect::EndTurn { p } | Effect::AfterAttackTriggers { p, .. } => p,
        _ => return Ok(()),
    };
    let pu = p as usize;
    if g.st.players[pu].marker.has_from(discarded(), me) {
        g.st.players[pu].marker.remove_from(discarded(), me);
        if g.st.players[pu].discard.iter().any(|c| c == me) {
            let a = g.st.players[pu].active;
            move_cards(g, ListRef::Discard(p), ListRef::Slot(p, a), &[me], me)?;
        }
    }
    Ok(())
}
