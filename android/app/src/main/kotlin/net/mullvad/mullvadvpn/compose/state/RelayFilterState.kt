package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider

data class RelayFilterState(
    val selectedOwnership: Ownership? = null,
    val allProviders: List<Provider> = emptyList(),
    val selectedProviders: List<Provider> = allProviders
) {
    val isApplyButtonEnabled = selectedProviders.isNotEmpty()

    val filteredOwnershipByProviders =
        if (selectedProviders.isEmpty()) {
            Ownership.entries
        } else {
            Ownership.entries.filter { ownership ->
                selectedProviders.any { provider ->
                    if (provider.mullvadOwned) {
                        ownership == Ownership.MullvadOwned
                    } else {
                        ownership == Ownership.Rented
                    }
                }
            }
        }
    val filteredProvidersByOwnership =
        when (selectedOwnership) {
            Ownership.MullvadOwned -> allProviders.filter { it.mullvadOwned }
            Ownership.Rented -> allProviders.filterNot { it.mullvadOwned }
            else -> allProviders
        }

    val isAllProvidersChecked = allProviders.size == selectedProviders.size
}
