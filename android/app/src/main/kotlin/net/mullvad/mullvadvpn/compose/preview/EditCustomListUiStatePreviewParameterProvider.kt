package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.EditCustomListUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

class EditCustomListUiStatePreviewParameterProvider :
    PreviewParameterProvider<EditCustomListUiState> {
    override val values =
        sequenceOf(
            EditCustomListUiState.Content(
                id = CustomListId("id"),
                name = CustomListName.fromString("Custom list"),
                locations =
                    listOf(
                        GeoLocationId.Hostname(
                            GeoLocationId.City(GeoLocationId.Country("country"), code = "city"),
                            "hostname",
                        )
                    ),
            ),
            EditCustomListUiState.Loading,
            EditCustomListUiState.NotFound,
        )
}
