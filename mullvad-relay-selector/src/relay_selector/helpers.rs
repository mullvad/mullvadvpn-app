//! This module contains various helper functions for the relay selector implementation.

use std::ops::Deref;

use mullvad_types::relay_list::Relay;
use rand::{Rng, seq::IteratorRandom};

/// Picks a relay at random from `relays`, but don't pick `exclude`.
pub fn pick_random_relay_excluding<'a, T>(relays: &'a [T], exclude: &'_ T) -> Option<&'a T>
where
    T: Deref<Target = Relay>,
{
    let filtered_relays = relays.iter().filter(|&a| a.deref() != exclude.deref());
    pick_random_relay_weighted(filtered_relays, |relay: &T| relay.weight)
}

/// Picks a relay using [pick_random_relay_weighted], using the `weight` member of each relay
/// as the weight function.
pub fn pick_random_relay<T>(relays: &[T]) -> Option<&T>
where
    T: Deref<Target = Relay>,
{
    pick_random_relay_weighted(relays.iter(), |relay| relay.weight)
}

/// Pick a random relay from the given slice. Will return `None` if the given slice is empty.
/// If all of the relays have a weight of 0, one will be picked at random without bias,
/// otherwise roulette wheel selection will be used to pick only relays with non-zero
/// weights.
pub fn pick_random_relay_weighted<'a, RelayType>(
    mut relays: impl Iterator<Item = &'a RelayType> + Clone,
    weight: impl Fn(&'a RelayType) -> u64,
) -> Option<&'a RelayType> {
    let total_weight: u64 = relays.clone().map(&weight).sum();
    let mut rng = rand::rng();
    if total_weight == 0 {
        relays.choose(&mut rng)
    } else {
        // Assign each relay a subset of the range 0..total_weight with size equal to its weight.
        // Pick a random number in the range 1..=total_weight. This chooses the relay with a
        // non-zero weight.
        //
        //                           rng(1..=total_weight)
        //                           |
        //                           v
        //   ________________________i_______________________________________________
        // 0|_____________|____________________|___________|_____|________|__________| total_weight
        //  ^             ^                    ^                          ^          ^
        //  |             |                    |                          |          |
        //  ------------------------------------                          ------------
        //         |                  |                                         |
        //   weight(relay 0)     weight(relay 1)    ..       ..     ..    weight(relay n)
        let mut i: u64 = rng.random_range(1..=total_weight);
        Some(
            relays
                .find(|relay| {
                    i = i.saturating_sub(weight(relay));
                    i == 0
                })
                .expect("At least one relay must've had a weight above 0"),
        )
    }
}
