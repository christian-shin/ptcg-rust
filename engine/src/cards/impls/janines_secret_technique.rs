//! Janine's Secret Art (SFA): choose up to 2 of your [D] Pokémon. For each of
//! those Pokémon, search your deck for a Basic [D] Energy card and attach it
//! to that Pokémon. Then, shuffle your deck. If you attached Energy to your
//! Active Pokémon in this way, it is now Poisoned.
//!
//! Twinleaf: throws when a Supporter was already played or the deck is empty (phase 4b); the card moves to
//! the Supporter area and the play is prevented; throws without a [D]
//! Pokémon in play; one AttachEnergyPrompt over the deck (Basic Energy named
//! 'Darkness Energy', non-[D] Pokémon blocked, different targets, 0-2, no
//! cancel). With no transfers only a shuffle follows; otherwise each Energy
//! is moved to its target (Poisoning the Active directly) and the shuffle
//! comes last.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "JaninesSecretTechnique", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    // Fixed (phase 4b, rulings 779/851): a search of an empty deck is not possible, so the card can't be played.
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);

    let mut dark = false;
    let mut blocked = SVec::new();
    for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).card_type.contains(&ct::DARK) {
            dark = true;
        } else {
            blocked.push(t);
        }
    }
    if !dark {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut o = AttachOpts::new(g.st.players[p].deck.len().min(255) as u8);
    o.allow_cancel = false;
    o.min = 0;
    o.max = 2;
    o.blocked_to = blocked;
    o.different_targets = true;
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    slots.push(SlotType::Active as u8);
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Darkness Energy"), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_ACTIVE",
        PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    if transfers.is_empty() {
        shuffle_deck(g, p);
        return Ok(());
    }
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Deck(p as u8), target.list(), &[c], me)?;
        if target.p as usize == p && target.s == g.st.players[p].active {
            crate::engine::phase::add_condition(&mut g.st.players[p].slots[target.s as usize], SpecialCondition::Poisoned);
        }
    }
    shuffle_deck(g, p);
    Ok(())
}
