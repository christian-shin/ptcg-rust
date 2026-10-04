//! Glass Trumpet (SCR): you can use this card only if you have any Tera
//! Pokémon in play. Choose up to 2 of your Benched [C] Pokémon and attach a
//! Basic Energy card from your discard pile to each of them.
//!
//! Twinleaf: the card goes hand→supporter (a no-op for an Item already
//! there) and the TrainerEffect is prevented before the checks. Fixed (phase
//! 4b): with no Benched [C] Pokémon the card throws CANNOT_PLAY_THIS_CARD
//! (the prompt below would have no legal target). AttachEnergyPrompt from the
//! discard: Bench only, min 1, max 2, different targets, blockedTo = every
//! non-[C] Pokémon (Active included).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GlassTrumpet", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let has_energy = g.st.players[p].discard.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.energy_type == EnergyType::Basic as u8
    });
    if !has_energy {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mons = for_each_pokemon(g, p, PlayerType::BottomPlayer);
    if !mons.iter().any(|(_, c, _)| g.st.cdef(*c).has_tag(tag::POKEMON_TERA)) {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if !mons.iter().any(|(_, c, t)| t.slot == SlotType::Bench && g.st.cdef(*c).card_type.contains(&ct::COLORLESS)) {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut blocked_to: SVec<CardTarget, 9> = SVec::new();
    for (_, c, t) in mons.iter().copied() {
        if !g.st.cdef(c).card_type.contains(&ct::COLORLESS) {
            blocked_to.push(t);
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let n = g.st.players[p].discard.len() as u8;
    let mut o = AttachOpts::new(n);
    o.allow_cancel = false;
    o.min = 1;
    o.max = 2;
    o.blocked_to = blocked_to;
    o.different_targets = true;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_BENCH",
        PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => return Ok(()),
    };
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
