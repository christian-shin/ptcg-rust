//! Decidueye (SFA 5): Stock Up on Feathers - draw cards until you have 7 cards in your hand. Power Shot - 170; discard a Basic [G]
//! Energy card from your hand. If you can't, this attack does nothing.
//!
//! Stock Up on Feathers is one Draw event of as many cards as are missing (none with 7 or more; this attack does no damage).
//! Power Shot chooses the Energy card before the damage and the damage reads that choice (`ChoiceDamageBy::NothingUnless`: with no
//! Basic [G] Energy card in hand nothing is chosen and the attack does no damage; otherwise the card is discarded after the damage;
//! Lurantis DRI 14, id2158).
use crate::spec::prelude::*;

const GRASS_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]);
const HAS_ONE: Cond = Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), GRASS_ENERGY);

pub static SPEC: CardSpec = CardSpec {
    class: "Decidueye",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(7)) }))] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Hand),
                    predicate: GRASS_ENERGY,
                    bounds: Bounds { min: Num::If(&HAS_ONE, &Num::Lit(1), &Num::Lit(0)), max: Num::If(&HAS_ONE, &Num::Lit(1), &Num::Lit(0)) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::ChoiceDamage(ChoiceDamageSpec { reg: Some(0), by: ChoiceDamageBy::NothingUnless(1) })),
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
