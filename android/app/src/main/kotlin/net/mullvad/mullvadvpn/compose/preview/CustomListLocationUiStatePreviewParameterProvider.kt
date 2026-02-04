package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsData
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.CheckableRelayListItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.generateRelayItemCountry
import net.mullvad.mullvadvpn.core.Lce

class CustomListLocationUiStatePreviewParameterProvider :
    PreviewParameterProvider<CustomListLocationsUiState> {
    override val values =
        sequenceOf(
            CustomListLocationsUiState(
                newList = true,
                content =
                    Lce.Content(
                        CustomListLocationsData(
                            locations =
                                listOf(
                                    CheckableRelayListItem(
                                        item =
                                            generateRelayItemCountry(
                                                name = "A relay",
                                                cityNames = listOf("City 1", "City 2"),
                                                relaysPerCity = 2,
                                                active = true,
                                            )
                                    ),
                                    CheckableRelayListItem(
                                        item =
                                            generateRelayItemCountry(
                                                    name = "Another relay",
                                                    cityNames =
                                                        listOf("City X", "City Y", "City Z"),
                                                    relaysPerCity = 1,
                                                    active = false,
                                                )
                                                .copy(id = GeoLocationId.Country("se"))
                                    ),
                                ),
                            searchTerm = "",
                            saveEnabled = true,
                            hasUnsavedChanges = true,
                        )
                    ),
            ),
            CustomListLocationsUiState(newList = false, content = Lce.Error(Unit)),
            CustomListLocationsUiState(newList = false, content = Lce.Loading(Unit)),
        )
}
