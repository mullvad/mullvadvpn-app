package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider

data class RelayFilterUiState(
    val selectedOwnership: Ownership? = null,
    val allProviders: List<Provider> = emptyList(),
    val selectedProviders: List<Provider> = allProviders,
) {
    val isApplyButtonEnabled = selectedProviders.isNotEmpty()

    val filteredOwnershipByProviders =
        if (selectedProviders.isEmpty()) {
            Ownership.entries
        } else {
            Ownership.entries.filter { ownership ->
                selectedProviders.any { provider -> provider.ownership == ownership }
            }
        }
    val filteredProvidersByOwnership =
        if (selectedOwnership == null) allProviders
        else allProviders.filter { provider -> provider.ownership == selectedOwnership }

    val isAllProvidersChecked = allProviders.size == selectedProviders.size
}
