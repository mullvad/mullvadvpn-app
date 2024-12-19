package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers

fun Ownership?.toOwnershipConstraint(): Constraint<Ownership> =
    when (this) {
        null -> Constraint.Any
        else -> Constraint.Only(this)
    }

fun Constraint<Providers>.toSelectedProviders(allProviders: List<Provider>): List<Provider> =
    when (this) {
        Constraint.Any -> allProviders
        is Constraint.Only ->
            value.providers.toList().mapNotNull { provider ->
                allProviders.firstOrNull { it.providerId == provider }
            }
    }

fun List<ProviderId>.toConstraintProviders(allProviders: List<ProviderId>): Constraint<Providers> =
    if (size == allProviders.size) {
        Constraint.Any
    } else {
        Constraint.Only(Providers(toHashSet()))
    }
