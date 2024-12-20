package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId

data class RelayFilterUiState(
    private val providerToOwnerships: Map<ProviderId, Set<Ownership>> = emptyMap(),
    val selectedOwnership: Constraint<Ownership> = Constraint.Any,
    val selectedProviders: Constraint<List<ProviderId>> = Constraint.Any,
) {
    val allProviders: List<ProviderId> = providerToOwnerships.keys.toList().sorted()

    val selectableOwnerships: List<Ownership> =
        when (selectedProviders) {
            Constraint.Any -> Ownership.entries
            is Constraint.Only ->
                if (selectedProviders.value.isEmpty()) {
                    Ownership.entries
                } else {
                    providerToOwnerships
                        .filterKeys { it in selectedProviders.value }
                        .values
                        .flatten()
                        .distinct()
                }
        }.sorted()

    val selectableProviders: List<ProviderId> =
        when (selectedOwnership) {
            Constraint.Any -> allProviders
            is Constraint.Only ->
                providerToOwnerships
                    .filterValues { selectedOwnership.value in it }
                    .keys
                    .toList()
                    .sorted()
        }

    val isApplyButtonEnabled = selectedProviders.getOrNull()?.isNotEmpty() != false

    val isAllProvidersChecked = selectedProviders is Constraint.Any
}
