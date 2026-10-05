//! Oricorio ex (M2): Excited Turbo — as often as you like during your turn,
//! if you have any [R] Mega Evolution Pokémon ex in play, attach a Basic [R]
//! Energy card from your hand to 1 of your Benched [R] Pokémon. Fire Wing — 110.
//!
//! Twinleaf: blockedTo lists every non-[R] Pokémon (Active included); the
//! prompt filter is `name: 'Fire Energy'`; each transfer is an
//! AttachEnergyEffect.
//!
//! Fixed (phase 4b, W4): the Mega ex check only looked at the cards stacked
//! in the Active Spot (now any Pokémon in play), and the card was missing its
//! `ex` tag (so a Knock Out gave 1 Prize card instead of 2).
//!
//! Fixed (phase 4b, R2): the AttachEnergyPrompt had the default min 0 / max
//! hand size, so a use could attach nothing (a free repeatable no-op) or many
//! cards; each use attaches exactly 1 Energy card (min 1, max 1).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Oricorioex", mask: mask(&[k::POWER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mega = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| {
            let d = g.st.cdef(*c);
            d.has_tag(tag::POKEMON_EX_LOWER) && d.has_tag(tag::POKEMON_SV_MEGA) && d.card_type.contains(&ct::FIRE)
        });
        if !mega {
            bail!("CANNOT_USE_POWER");
        }
        let fire_bench = g.st.players[p].bench.iter().any(|s| {
            g.st.players[p].slots[*s as usize].cards.iter().any(|c| {
                let d = g.st.cdef(c);
                d.is_pokemon() && d.card_type.contains(&ct::FIRE)
            })
        });
        if !fire_bench {
            bail!("CANNOT_USE_POWER");
        }
        let mut blocked_to: SVec<CardTarget, 9> = SVec::new();
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if !g.st.cdef(c).card_type.contains(&ct::FIRE) {
                blocked_to.push(t);
            }
        }
        let has_energy = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::FIRE)
        });
        if !has_energy {
            bail!("CANNOT_USE_POWER");
        }
        let mut o = AttachOpts::new(g.st.players[p].hand.len() as u8);
        o.allow_cancel = false;
        o.min = 1;
        o.max = 1;
        o.blocked_to = blocked_to;
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Fire Energy"),
            ..Filter::none()
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_CARDS",
            PromptKind::AttachEnergy { cards: ListRef::Hand(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
    }
    Ok(())
}
