package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId

data class RelayFilterUiState(
    private val providerToOwnerships: Map<ProviderId, Set<Ownership>> = emptyMap(),
    val selectedOwnership: Ownership? = null,
    val selectedProviders: List<ProviderId> = emptyList(),
) {
    val allProviders: List<ProviderId> = providerToOwnerships.keys.toList().sorted()

    val selectableOwnerships: List<Ownership> =
        if (selectedProviders.isEmpty()) {
                Ownership.entries
            } else {
                providerToOwnerships
                    .filterKeys { it in selectedProviders }
                    .values
                    .flatten()
                    .distinct()
            }
            .sorted()

    val selectableProviders: List<ProviderId> =
        if (selectedOwnership != null) {
            providerToOwnerships.filterValues { selectedOwnership in it }.keys.toList().sorted()
        } else {
            allProviders
        }

    val removedProviders: List<ProviderId> = selectedProviders - allProviders

    val isApplyButtonEnabled = selectedProviders.isNotEmpty()

    val isAllProvidersChecked = allProviders.size == selectedProviders.size
}
