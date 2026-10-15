//! Ogre's Mask (PRE 118, Item): choose a Pokémon ex in your discard pile that
//! has "Ogerpon" in its name, and switch it with 1 of your Pokémon ex in play
//! that has "Ogerpon" in its name. Attached cards, damage counters, Special
//! Conditions, turns in play and other effects remain on the new Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "OgresMask",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonEx, Pred::NameContains("Ogerpon")]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Top(Pred::All(&[Pred::Pokemon, Pred::PokemonEx, Pred::NameContains("Ogerpon")]))) }, purpose: PromptPurpose::SwapPokemon })),
            Step::new(Op::SwapPokemonCard(SwapPokemonCardSpec { cards: 0, slot: SlotExpr::Picked, into: ZoneRef(Who::Me, Zone::Discard), keep_index: true, bottom: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
