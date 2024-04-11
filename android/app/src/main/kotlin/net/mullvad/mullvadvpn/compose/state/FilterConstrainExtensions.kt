package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.Providers

fun Constraint<Ownership>.toNullableOwnership(): Ownership? =
    when (this) {
        Constraint.Any -> null
        is Constraint.Only -> this.value
    }

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

fun List<Provider>.toConstraintProviders(allProviders: List<Provider>): Constraint<Providers> =
    if (size == allProviders.size) {
        Constraint.Any
    } else {
        Constraint.Only(Providers(map { provider -> provider.providerId }.toHashSet()))
    }
