package net.mullvad.mullvadvpn.filter.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId

private val PROVIDER_TO_OWNERSHIPS = mapOf(ProviderId("provider1") to setOf(Ownership.MullvadOwned))

class FilterUiStatePreviewParameterProvider : PreviewParameterProvider<FilterUiState> {
    override val values =
        sequenceOf(
            FilterUiState(
                providerToOwnerships = PROVIDER_TO_OWNERSHIPS,
                selectedOwnership = Constraint.Only(Ownership.MullvadOwned),
                selectedProviders = Constraint.Only(PROVIDER_TO_OWNERSHIPS.keys),
            )
        )
}
