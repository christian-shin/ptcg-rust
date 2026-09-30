//! Festival Grounds (TWM, stadium): each Pokémon with any Energy attached
//! recovers from all Special Conditions and can't be affected by any.
//!
//! Twinleaf: implemented as a CheckTableStateEffect sweep (conditions are
//! cleared whenever the table state is checked, not prevented). The stadium
//! can't be "used". Festival Lead's double attack lives on the Festival Lead
//! cards (runtime `barrage`, see Dipplin TWM) and the core useAttack.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "FestivalGrounds",
    mask: mask(&[k::CHECK_TABLE_STATE, k::USE_STADIUM]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckTableState { .. } if g.st.stadium_card() == Some(me) => {
            for p in 0..2usize {
                for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                    let slot = g.st.slot(p, s);
                    if slot.special_conditions.is_empty() || slot.energies.is_empty() {
                        continue;
                    }
                    if is_stadium_effect_blocked(g, p, SlotRef::new(p, s), me) {
                        continue;
                    }
                    // clearAllSpecialConditions(): removes the five conditions.
                    let sc = &mut g.st.players[p].slots[s as usize].special_conditions;
                    for c in [SpecialCondition::Poisoned, SpecialCondition::Asleep, SpecialCondition::Burned, SpecialCondition::Confused, SpecialCondition::Paralyzed] {
                        sc.retain(|x| *x != c as u8);
                    }
                }
            }
            Ok(())
        }
        Effect::UseStadium { .. } if g.st.stadium_card() == Some(me) => bail!("CANNOT_USE_STADIUM"),
        _ => Ok(()),
    }
}
