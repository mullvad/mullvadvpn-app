package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.compose.state.RelayLocationListItem
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

class CustomListLocationUiStatePreviewParameterProvider :
    PreviewParameterProvider<CustomListLocationsUiState> {
    override val values =
        sequenceOf(
            CustomListLocationsUiState.Content.Data(
                newList = true,
                locations =
                    listOf(
                        RelayLocationListItem(
                            item =
                                RelayItemPreviewData.generateRelayItemCountry(
                                    name = "A relay",
                                    cityNames = listOf("City 1", "City 2"),
                                    relaysPerCity = 2,
                                    active = true,
                                )
                        ),
                        RelayLocationListItem(
                            item =
                                RelayItemPreviewData.generateRelayItemCountry(
                                        name = "Another relay",
                                        cityNames = listOf("City X", "City Y", "City Z"),
                                        relaysPerCity = 1,
                                        active = false,
                                    )
                                    .copy(id = GeoLocationId.Country("se"))
                        ),
                    ),
                searchTerm = "",
                saveEnabled = true,
                hasUnsavedChanges = true,
            ),
            CustomListLocationsUiState.Content.Empty(
                newList = false,
                searchTerm = "searchTerm",
                isSearching = true,
            ),
            CustomListLocationsUiState.Loading(newList = true),
        )
}
