//! Mega Manectric ex (M1S): Flash Ray — 120; during your opponent's next
//! turn, prevent all damage done to this Pokémon by attacks from Basic
//! Pokémon. Riotous Blasting — 200+; you may discard all Energy from this
//! Pokémon for 130 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaManectricEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Stage(crate::types::Stage::Basic)) }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_DISCARD_ENERGY",
                yes: &[
                    Step::new(more_damage_if(130, Cond::True)),
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::This, selection: EnergySelection::AllProvided })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Kept for Ceruledge ex (still hand-written); delete with its conversion.
use crate::cards::prelude::*;

/// `DISCARD_ALL_ENERGY_FROM_POKEMON(store, state, effect, card)`.
pub fn discard_all_energy_from_pokemon(g: &mut Game, e: EffId, card: CardId) -> R {
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let (mp, ms) = match g.st.find_pokemon_slot(card) {
        Some(x) => x,
        None => bail!("INVALID_TARGET"),
    };
    let pu = p as usize;
    let active = SlotRef::new(pu, g.st.players[pu].active);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
    let mut cards: SVec<CardId, 64> = SVec::new();
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for em in energy_map.iter() {
            cards.push(em.card);
        }
    }
    let target = SlotRef::new(mp, ms);
    g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }, cards })?;
    Ok(())
}

