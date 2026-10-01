//! Reboot Pod (TEF, ACE SPEC): attach a Basic Energy card from your discard
//! pile to each of your Future Pokémon in play.
//!
//! Twinleaf: one AttachEnergyPrompt from the discard (0..basic-energy-count,
//! no cancel, different targets, non-Future Pokémon blocked). Each transfer
//! is a plain MOVE_CARDS discard→slot (no AttachEnergyEffect) followed by a
//! MOVE_CARDS of the card supporter→discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RebootPod", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let n = g.st.players[p]
        .discard
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        })
        .count();
    if n == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut blocked_to = SVec::new();
    for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if !g.st.cdef(c).has_tag(tag::FUTURE) {
            blocked_to.push(t);
        }
    }
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    slots.push(SlotType::Active as u8);
    let mut o = AttachOpts::new(n as u8);
    o.allow_cancel = false;
    o.min = 0;
    o.max = n as u8;
    o.blocked_to = blocked_to;
    o.different_targets = true;
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_ACTIVE",
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
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    if transfers.is_empty() {
        move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
        return Ok(());
    }
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
        move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    }
    Ok(())
}
