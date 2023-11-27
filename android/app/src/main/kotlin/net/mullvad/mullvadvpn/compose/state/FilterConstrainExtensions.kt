package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.relaylist.Provider

fun Constraint<Ownership>.toNullableOwnership(): Ownership? =
    when (this) {
        is Constraint.Any -> null
        is Constraint.Only -> this.value
    }

fun Ownership?.toOwnershipConstraint(): Constraint<Ownership> =
    when (this) {
        null -> Constraint.Any()
        else -> Constraint.Only(this)
    }

fun Constraint<Providers>.toSelectedProviders(allProviders: List<Provider>): List<Provider> =
    when (this) {
        is Constraint.Any -> allProviders
        is Constraint.Only ->
            this.value.providers.toList().mapNotNull { providerName ->
                allProviders.firstOrNull { it.name == providerName }
            }
    }

fun List<Provider>.toConstraintProviders(allProviders: List<Provider>): Constraint<Providers> =
    if (size == allProviders.size) {
        Constraint.Any()
    } else {
        Constraint.Only(Providers(map { provider -> provider.name }.toHashSet()))
    }
