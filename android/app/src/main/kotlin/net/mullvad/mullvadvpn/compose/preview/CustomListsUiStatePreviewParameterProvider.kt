package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

class CustomListsUiStatePreviewParameterProvider : PreviewParameterProvider<CustomListsUiState> {
    override val values =
        sequenceOf(
            CustomListsUiState.Content(
                customLists =
                    listOf(
                        CustomList(
                            id = CustomListId("list1"),
                            name = CustomListName.fromString("Custom List 1"),
                            locations =
                                listOf(
                                    GeoLocationId.Hostname(
                                        city =
                                            GeoLocationId.City(
                                                country = GeoLocationId.Country("se"),
                                                code = "code",
                                            ),
                                        code = "code",
                                    )
                                ),
                        ),
                        CustomList(
                            id = CustomListId("list2"),
                            name = CustomListName.fromString("Custom List 2"),
                            locations =
                                listOf(
                                    GeoLocationId.Hostname(
                                        city =
                                            GeoLocationId.City(
                                                country = GeoLocationId.Country("de"),
                                                code = "code",
                                            ),
                                        code = "code",
                                    )
                                ),
                        ),
                    )
            ),
            CustomListsUiState.Content(),
            CustomListsUiState.Loading,
        )
}
