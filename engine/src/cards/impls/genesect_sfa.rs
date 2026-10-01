//! Genesect (SFA 40): Ace Canceller — if this Pokémon has a Pokémon Tool
//! attached, your opponent can't play any ACE SPEC cards from their hand.
//! Magnet Blast — 100.
//!
//! Twinleaf: PlayItemEffect / AttachPokemonToolEffect / AttachEnergyEffect /
//! PlayStadiumEffect whose card has the ACE SPEC tag: when this card is the
//! top Pokémon of a slot with a tool on the playing player's opponent's side,
//! the ability-lock probe runs for the *playing* player, and
//! BLOCKED_BY_EFFECT is thrown when it passes. (AttachEnergyEffect fires for
//! any energy attachment, not only from the hand.)
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Genesect@SFA",
    mask: mask(&[k::PLAY_ITEM, k::ATTACH_POKEMON_TOOL, k::ATTACH_ENERGY, k::PLAY_STADIUM]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, card) = match *g.e(e) {
        Effect::PlayItem { p, card, .. }
        | Effect::AttachPokemonTool { p, card, .. }
        | Effect::AttachEnergy { p, card, .. }
        | Effect::PlayStadium { p, card } => (p as usize, card),
        _ => return Ok(()),
    };
    if !g.st.cdef(card).has_tag(tag::ACE_SPEC) {
        return Ok(());
    }
    let o = 1 - p;
    let active = for_each_pokemon(g, o, PlayerType::TopPlayer).iter().any(|(s, c, _)| *c == me && !g.st.slot(o, *s).tools.is_empty());
    if !active {
        return Ok(());
    }
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    bail!("BLOCKED_BY_EFFECT");
}
