//! Blowtorch (PFL): Item. You can use this card only if you discard a Basic [R] Energy card from your hand.
//! Discard a Pokémon Tool or Special Energy card from 1 of your opponent's Pokémon, or discard a Stadium in play.
//!
//! The Energy is the cost: it is picked and discarded first. The choice that follows lists only the kinds that exist
//! (a Tool, a Special Energy card, a Stadium). With several Tools on the Pokémon, one Tool is chosen and only it is
//! discarded. No damage, counters or Knock Out here: nothing changed in events batch 6.
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);
const FIRE_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")]);
const OPP_POKEMON: SlotSel = SlotSel::Pokemon(Who::Opp);
const MY_STADIUM: ZoneRef = ZoneRef(Who::Me, Zone::Stadium);
const OPP_STADIUM: ZoneRef = ZoneRef(Who::Opp, Zone::Stadium);
const HAS_TOOL: Cond = Cond::AnySlot(OPP_POKEMON, SlotPred::AnyTool(Pred::Any));
const HAS_SPECIAL: Cond = Cond::AnySlot(OPP_POKEMON, SlotPred::HasSpecialEnergy);
const STADIUM_IN_PLAY: Cond = Cond::Any(&[Cond::Nonempty(MY_STADIUM, Pred::Any), Cond::Nonempty(OPP_STADIUM, Pred::Any)]);
const ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::Attached(SlotExpr::Picked));
const TOOLS: ZoneRef = ZoneRef(Who::Opp, Zone::Tools(SlotExpr::Picked));

pub static SPEC: CardSpec = CardSpec {
    class: "Blowtorch",
    // Discard a Basic [R] Energy card from your hand to use this card. Discard a Pokémon Tool or
    // Special Energy card from 1 of your opponent's Pokémon, or discard a Stadium in play.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Nonempty(HAND, FIRE_ENERGY), Cond::Any(&[HAS_TOOL, HAS_SPECIAL, STADIUM_IN_PLAY])],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: HAND, predicate: FIRE_ENERGY, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: HAND, to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Choose(ChooseSpec {
                chooser: Who::Me,
                msg: "DISCARD_STADIUM_OR_TOOL_OR_SPECIAL_ENERGY",
                options: &[
                    ChoiceBranch {
                        label: "CHOICE_TOOL",
                        avail: HAS_TOOL,
                        body: &[
                            Step::new(Op::PickSlot(PickSlotSpec {
                                chooser: Who::Me,
                                among: SlotSel::Filtered(&OPP_POKEMON, SlotPred::AnyTool(Pred::Any)),
                                msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS",
                            })),
                            // One Tool of the Pokémon, chosen when it has several.
                            Step::new(Op::Pick(PickSpec { from: TOOLS, predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
                            Step::new(Op::Move(MoveSpec { from: TOOLS, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
                        ],
                    },
                    ChoiceBranch {
                        label: "CHOICE_SPECIAL_ENERGY",
                        avail: HAS_SPECIAL,
                        body: &[
                            Step::new(Op::PickSlot(PickSlotSpec {
                                chooser: Who::Me,
                                among: SlotSel::Filtered(&OPP_POKEMON, SlotPred::HasSpecialEnergy),
                                msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS",
                            })),
                            Step::new(Op::Pick(PickSpec {
                                from: ATTACHED,
                                predicate: Pred::SpecialEnergy,
                                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                                into: 0,
                                msg: "CHOOSE_CARD_TO_DISCARD",
                                ..PickSpec::DEFAULT
                            })),
                            Step::new(Op::Move(MoveSpec { from: ATTACHED, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
                        ],
                    },
                    ChoiceBranch {
                        label: "CHOICE_STADIUM",
                        avail: STADIUM_IN_PLAY,
                        body: &[Step::new(DISCARD_STADIUM)],
                    },
                ],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
