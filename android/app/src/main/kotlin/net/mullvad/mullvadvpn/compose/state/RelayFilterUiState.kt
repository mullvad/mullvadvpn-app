package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId

data class RelayFilterUiState(
    val filteredOwnershipByProviders: List<Ownership> = Ownership.entries,
    val selectedOwnership: Ownership? = null,
    val filteredProvidersByOwnership: List<ProviderId> = listOf(),
    val allProviders: List<ProviderId> = emptyList(),
    val selectedProviders: List<ProviderId> = filteredProvidersByOwnership,
) {
    val isApplyButtonEnabled = selectedProviders.isNotEmpty()

    val isAllProvidersChecked = allProviders.size == selectedProviders.size
}
