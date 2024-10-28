package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.ModelOwnership

private val RELAY =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(country = GeoLocationId.Country("se"), code = "code"),
                code = "code",
            ),
        provider = Provider(providerId = ProviderId("providerId"), ownership = Ownership.Rented),
        active = true,
        daita = true,
    )

class SelectLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<SelectLocationUiState> {
    override val values =
        sequenceOf(
            SelectLocationUiState(
                // searchTerm = "search term",
                listOf(FilterChip.Ownership(ownership = ModelOwnership.MullvadOwned)),
                multihopEnabled = true,
                relayListType = RelayListType.ENTRY,
            )
        )
}
