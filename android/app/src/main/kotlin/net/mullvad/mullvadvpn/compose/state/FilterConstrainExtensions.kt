package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers

fun Ownership?.toOwnershipConstraint(): Constraint<Ownership> =
    when (this) {
        null -> Constraint.Any
        else -> Constraint.Only(this)
    }

fun Providers.toConstraintProviders(allProviders: Providers): Constraint<Providers> =
    if (size == allProviders.size) {
        Constraint.Any
    } else {
        Constraint.Only(this)
    }
