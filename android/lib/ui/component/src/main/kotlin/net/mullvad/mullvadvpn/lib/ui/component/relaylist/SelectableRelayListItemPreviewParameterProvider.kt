package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import androidx.compose.ui.tooling.preview.PreviewParameterProvider

class SelectableRelayListItemPreviewParameterProvider :
    PreviewParameterProvider<List<RelayListItem.SelectableItem>> {
    override val values =
        sequenceOf(
            listOf(
                RelayListItem.GeoLocationItem(
                    item =
                        generateRelayItemCountry(
                            name = "Relay country Active",
                            cityNames = listOf("Relay city 1", "Relay city 2"),
                            relaysPerCity = 2,
                        ),
                    isSelected = true,
                    expanded = false,
                    itemPosition = ItemPosition.Single,
                ),
                RelayListItem.GeoLocationItem(
                    item =
                        generateRelayItemCountry(
                            name = "Not Enabled Relay country",
                            cityNames = listOf("Not Enabled city"),
                            relaysPerCity = 1,
                            active = false,
                        ),
                    isSelected = false,
                    itemPosition = ItemPosition.Single,
                ),
                RelayListItem.GeoLocationItem(
                    item =
                        generateRelayItemCountry(
                            name = "Relay country Expanded",
                            cityNames = listOf("Normal city"),
                            relaysPerCity = 2,
                        ),
                    isSelected = true,
                    expanded = true,
                    itemPosition = ItemPosition.Single,
                ),
                RelayListItem.GeoLocationItem(
                    item =
                        generateRelayItemCountry(
                            name = "Country and city Expanded",
                            cityNames = listOf("Expanded city A", "Expanded city B"),
                            relaysPerCity = 2,
                        ),
                    isSelected = false,
                    itemPosition = ItemPosition.Single,
                ),
                RelayListItem.GeoLocationItem(
                    item =
                        generateRelayItemCountry(
                            name = "Country selected but inactive",
                            cityNames = listOf("Expanded city A", "Expanded city B"),
                            relaysPerCity = 2,
                            active = false,
                        ),
                    isSelected = true,
                    itemPosition = ItemPosition.Single,
                ),
            )
        )
}
