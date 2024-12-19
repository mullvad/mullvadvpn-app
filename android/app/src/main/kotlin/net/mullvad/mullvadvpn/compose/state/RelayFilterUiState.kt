package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId

data class RelayFilterUiState(
    private val providerOwnershipMap: Map<ProviderId, Set<Ownership>> = emptyMap(),
    val selectedOwnership: Ownership? = null,
    val selectedProviders: List<ProviderId> = emptyList(),
) {
    val selectableOwnerships: List<Ownership> =
        if (selectedProviders.isEmpty()) {
                Ownership.entries
            } else {
                providerOwnershipMap
                    .filterKeys { it in selectedProviders }
                    .values
                    .flatten()
                    .distinct()
            }
            .sorted()

    val selectableProviders: List<ProviderId> =
        if (selectedOwnership != null)
            providerOwnershipMap.filterValues { selectedOwnership in it }.keys.toList()
        else providerOwnershipMap.keys.toList()

    val allProviders: List<ProviderId> = providerOwnershipMap.keys.toList()

    val isApplyButtonEnabled = selectedProviders.isNotEmpty()

    val isAllProvidersChecked = allProviders.size == selectedProviders.size
}
