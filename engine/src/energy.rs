//! Energy payment checks (`StateUtils.checkEnoughEnergy` and friends).

use crate::effects::{Cost, EnergyEntry, EnergyMap};
use crate::list::SVec;
use crate::types::{ct, CardType};

pub fn is_choosable(provides: &[CardType]) -> bool {
    let mut first = None;
    for &p in provides {
        match first {
            None => first = Some(p),
            Some(f) if f != p => return true,
            _ => {}
        }
    }
    false
}

pub fn unit_count(provides: &[CardType]) -> usize {
    if is_choosable(provides) {
        1
    } else {
        provides.len()
    }
}

pub fn provides_matches(provides: &[CardType], t: CardType) -> bool {
    provides.contains(&ct::ANY) || provides.contains(&t)
}

#[derive(Clone, Copy, Default)]
struct Unit {
    choosable: bool,
    t: CardType,
    types: SVec<CardType, 4>,
}

fn units(energy: &[EnergyEntry]) -> SVec<Unit, 192> {
    let mut out = SVec::new();
    for e in energy {
        let p = e.provides.as_slice();
        if is_choosable(p) {
            let mut types = SVec::new();
            for &t in p {
                types.push(t);
            }
            out.push(Unit { choosable: true, t: 0, types });
        } else {
            for &t in p {
                out.push(Unit { choosable: false, t, types: SVec::new() });
            }
        }
    }
    out
}

pub fn check_enough_energy(energy: &[EnergyEntry], cost: &[CardType]) -> bool {
    if cost.is_empty() {
        return true;
    }
    let mut us = units(energy);
    let mut colorless = 0usize;
    let mut needs: SVec<CardType, 16> = SVec::new();
    for &c in cost {
        match c {
            ct::ANY | ct::NONE => {}
            ct::COLORLESS => colorless += 1,
            _ => {
                if let Some(i) = us.iter().position(|u| !u.choosable && u.t == c) {
                    us.remove_at(i);
                } else {
                    needs.push(c);
                }
            }
        }
    }
    let mut i = 0;
    while i < needs.len() {
        let n = *needs.get(i).unwrap();
        if let Some(j) = us.iter().position(|u| u.choosable && u.types.contains(&n)) {
            us.remove_at(j);
            needs.remove_at(i);
        } else {
            i += 1;
        }
    }
    for _ in 0..needs.len() {
        if let Some(j) = us.iter().position(|u| !u.choosable && u.t == ct::ANY) {
            us.remove_at(j);
        } else {
            return false;
        }
    }
    us.len() >= colorless
}

pub fn check_exact_energy(energy: &[EnergyEntry], cost: &[CardType]) -> bool {
    if !check_enough_energy(energy, cost) {
        return false;
    }
    for i in 0..energy.len() {
        let mut tmp: SVec<EnergyEntry, 64> = SVec::new();
        for (j, e) in energy.iter().enumerate() {
            if j != i {
                tmp.push(*e);
            }
        }
        if check_enough_energy(tmp.as_slice(), cost) {
            return false;
        }
    }
    true
}

/// `StateUtils.checkEnergyPayment`: whether `energy` is an allowed payment for an Energy cost that is discarded or paid by
/// choosing Energy cards (Retreat Cost, "discard N Energy"). Ruling 1652 (retreat; Advanced Rulebook C-01 for Ignition
/// Energy): a number of cards equal to the cost, or a sequence of discards that first met the cost with its last card;
/// never more cards than the cost.
pub fn check_energy_payment(energy: &[EnergyEntry], cost: &[CardType]) -> bool {
    if !check_enough_energy(energy, cost) {
        return false;
    }
    if check_exact_energy(energy, cost) {
        return true;
    }
    if energy.len() > cost.len() {
        return false;
    }
    if energy.len() == cost.len() {
        return true;
    }
    (0..energy.len()).any(|i| {
        let tmp: Vec<EnergyEntry> = energy.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, e)| *e).collect();
        !check_enough_energy(&tmp, cost)
    })
}

pub fn all_provides_identical(map: &[EnergyEntry]) -> bool {
    if map.is_empty() {
        return false;
    }
    let key = |e: &EnergyEntry| {
        let mut v: Vec<CardType> = e.provides.as_slice().to_vec();
        // JS default sort compares string forms; single-digit types sort the same.
        v.sort_by_key(|x| x.to_string());
        v
    };
    let first = key(&map[0]);
    map.iter().all(|e| key(e) == first)
}

fn apply_provides_to_typed_costs(provides: &[CardType], costs: &mut SVec<CardType, 16>) {
    if is_choosable(provides) {
        if let Some(i) = costs.iter().position(|c| provides_matches(provides, *c)) {
            costs.remove_at(i);
        }
        return;
    }
    for &c in provides {
        if c == ct::ANY && !costs.is_empty() {
            costs.remove_at(0);
        } else if let Some(i) = costs.position(&c) {
            costs.remove_at(i);
        }
    }
}

/// `StateUtils.selectMinimalEnergyForCost`.
pub fn select_minimal_energy_for_cost(map: &[EnergyEntry], cost: &[CardType]) -> Option<SVec<EnergyEntry, 64>> {
    let mut result: SVec<EnergyEntry, 64> = SVec::new();
    if cost.is_empty() {
        return Some(result);
    }
    let mut provides: Vec<EnergyEntry> = map.to_vec();
    let mut costs: SVec<CardType, 16> = SVec::new();
    for &c in cost {
        if c != ct::COLORLESS {
            costs.push(c);
        }
    }
    while !costs.is_empty() && !provides.is_empty() {
        let t = *costs.get(0).unwrap();
        let mut idx = provides.iter().position(|p| provides_matches(p.provides.as_slice(), t));
        if idx.is_none() {
            idx = provides.iter().position(|p| p.provides.contains(&ct::ANY));
        }
        let i = idx?;
        let p = provides.remove(i);
        result.push(p);
        apply_provides_to_typed_costs(p.provides.as_slice(), &mut costs);
    }
    if !costs.is_empty() {
        return None;
    }
    // Stable sort by unit count (Array.prototype.sort is stable).
    provides.sort_by_key(|p| unit_count(p.provides.as_slice()));
    let mut k = 0;
    while k < provides.len() && !check_enough_energy(result.as_slice(), cost) {
        result.push(provides[k]);
        k += 1;
    }
    if !check_enough_energy(result.as_slice(), cost) {
        return None;
    }
    loop {
        let mut changed = false;
        for i in 0..result.len() {
            let mut tmp: SVec<EnergyEntry, 64> = SVec::new();
            for (j, e) in result.iter().enumerate() {
                if j != i {
                    tmp.push(*e);
                }
            }
            if check_enough_energy(tmp.as_slice(), cost) {
                result = tmp;
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
    Some(result)
}

/// `ChooseEnergyPrompt.getCostThatCanBePaid` (used when the prompt can't be cancelled).
pub fn cost_that_can_be_paid(energy: &EnergyMap, cost: &Cost) -> Cost {
    let mut result = *cost;
    let mut provides: Vec<EnergyEntry> = energy.as_slice().to_vec();
    let mut costs: SVec<CardType, 16> = SVec::new();
    for &c in cost.iter() {
        if c != ct::COLORLESS {
            costs.push(c);
        }
    }
    let colorless_count = result.len() - costs.len();
    while !costs.is_empty() && !provides.is_empty() {
        let c = *costs.get(0).unwrap();
        let mut idx = provides.iter().position(|p| provides_matches(p.provides.as_slice(), c));
        if idx.is_none() {
            idx = provides.iter().position(|p| p.provides.contains(&ct::ANY));
        }
        match idx {
            Some(i) => {
                let p = provides.remove(i);
                let pr = p.provides.as_slice();
                if is_choosable(pr) {
                    if let Some(m) = costs.iter().position(|x| provides_matches(pr, *x)) {
                        costs.remove_at(m);
                    }
                } else {
                    for &x in pr {
                        if x == ct::ANY && !costs.is_empty() {
                            costs.remove_at(0);
                        } else if let Some(m) = costs.position(&x) {
                            costs.remove_at(m);
                        }
                    }
                }
            }
            None => {
                costs.remove_at(0);
                if let Some(d) = result.position(&c) {
                    result.remove_at(d);
                }
            }
        }
    }
    let mut left = 0usize;
    for p in &provides {
        left += unit_count(p.provides.as_slice());
    }
    let to_delete = colorless_count.saturating_sub(left);
    for _ in 0..to_delete {
        if let Some(d) = result.position(&ct::COLORLESS) {
            result.remove_at(d);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(card: u8, t: CardType) -> EnergyEntry {
        let mut provides = SVec::new();
        provides.push(t);
        EnergyEntry { card, provides }
    }

    /// One Pokemon can hold more than 40 Energy cards (a deck holds 60); none of the Energy checks may overflow.
    #[test]
    fn more_than_forty_energy_on_one_pokemon() {
        let mut map: EnergyMap = SVec::new();
        for i in 0..58u8 {
            map.push(entry(i, ct::FIRE));
        }
        let cost = [ct::FIRE, ct::COLORLESS];
        assert!(check_enough_energy(map.as_slice(), &cost));
        let sel = select_minimal_energy_for_cost(map.as_slice(), &cost).unwrap();
        assert_eq!(sel.len(), 2);
        assert!(check_exact_energy(sel.as_slice(), &cost));
        assert!(!check_exact_energy(map.as_slice(), &cost));
        let mut big_cost: Cost = SVec::new();
        for _ in 0..16 {
            big_cost.push(ct::COLORLESS);
        }
        let paid = cost_that_can_be_paid(&map, &big_cost);
        assert_eq!(paid.len(), 16);
        let sel = select_minimal_energy_for_cost(map.as_slice(), big_cost.as_slice()).unwrap();
        assert_eq!(sel.len(), 16);
    }
}

#[cfg(test)]
mod payment_tests {
    use super::*;
    use crate::list::CardId;

    fn entry(card: CardId, n: usize) -> EnergyEntry {
        let mut provides = SVec::new();
        for _ in 0..n {
            provides.push(ct::COLORLESS);
        }
        EnergyEntry { card, provides }
    }

    fn cost(n: usize) -> Vec<CardType> {
        vec![ct::COLORLESS; n]
    }

    /// Ruling 1652: a cost of 2 may be paid with 1 or 2 Energy cards that provide 2 each, never more cards than the cost.
    #[test]
    fn several_unit_cards_pay_with_one_card_or_as_many_as_the_cost() {
        let (a, b, c) = (entry(1, 2), entry(2, 2), entry(3, 2));
        assert!(check_energy_payment(&[a], &cost(2)));
        assert!(check_energy_payment(&[a, b], &cost(2)));
        assert!(!check_energy_payment(&[a, b, c], &cost(2)));
        assert!(!check_energy_payment(&[], &cost(2)));
    }

    /// C-01: Ignition Energy (3 units) pays a cost of 3 alone, or with 3 cards in all; 2 cards of which one is left over do not.
    #[test]
    fn ignition_energy_pays_alone_or_with_three_cards() {
        let (i1, i2, f) = (entry(1, 3), entry(2, 3), entry(3, 1));
        assert!(check_energy_payment(&[i1], &cost(3)));
        assert!(check_energy_payment(&[i1, i2, f], &cost(3)));
        assert!(!check_energy_payment(&[i1, i2], &cost(3)));
    }

    /// Ruling 1652 for "discard 3 [M] Energy" (Metagross CRI) and "discard 3 Energy" (Explosion Y): Neo Upper Energy on a
    /// Stage 2 (2 of any type) and a Metal Energy pay [M][M][M] with 2 cards; Water + Ignition pay a 3 (until fulfilled);
    /// a fourth card is never allowed; Ignition Energy provides [C] only and cannot pay a typed cost.
    #[test]
    fn typed_and_mixed_payments_follow_the_ruling() {
        let mk = |card: CardId, v: &[CardType]| {
            let mut provides = SVec::new();
            for t in v {
                provides.push(*t);
            }
            EnergyEntry { card, provides }
        };
        let neo = mk(1, &[ct::ANY, ct::ANY]);
        let (m1, m2, m3) = (mk(2, &[ct::METAL]), mk(3, &[ct::METAL]), mk(4, &[ct::METAL]));
        let ign = mk(5, &[ct::COLORLESS, ct::COLORLESS, ct::COLORLESS]);
        let water = mk(6, &[ct::WATER]);
        let mmm = [ct::METAL, ct::METAL, ct::METAL];
        assert!(check_energy_payment(&[neo, m1], &mmm));
        assert!(check_energy_payment(&[neo, m1, m2], &mmm));
        assert!(!check_energy_payment(&[neo, m1, m2, m3], &mmm));
        assert!(!check_energy_payment(&[m1, m2, m3, ign], &mmm));
        assert!(!check_energy_payment(&[m1, m2, ign], &mmm));
        assert!(!check_energy_payment(&[neo], &mmm));
        assert!(check_energy_payment(&[water, ign], &cost(3)));
        assert!(check_energy_payment(&[ign], &cost(3)));
        assert!(check_energy_payment(&[ign], &cost(2)));
        assert!(!check_energy_payment(&[water, ign, m1], &cost(2)));
        assert!(!check_energy_payment(&[water, m1, m2], &cost(2)));
    }
}
