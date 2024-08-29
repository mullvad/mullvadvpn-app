package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.preview.RelayItemPreviewData.generateRelayItemCountry
import net.mullvad.mullvadvpn.lib.model.RelayItem

class RelayItemCheckableCellPreviewParameterProvider :
    PreviewParameterProvider<List<RelayItem.Location.Country>> {
    override val values =
        sequenceOf(
            listOf(
                generateRelayItemCountry(
                    name = "Relay country Active",
                    cityNames = listOf("Relay city 1", "Relay city 2"),
                    relaysPerCity = 2,
                ),
                generateRelayItemCountry(
                    name = "Relay country Expanded",
                    cityNames = listOf("Normal city"),
                    relaysPerCity = 2,
                ),
                generateRelayItemCountry(
                    name = "Country and city Expanded",
                    cityNames = listOf("Expanded city A", "Expanded city B"),
                    relaysPerCity = 2,
                ),
            )
        )
}
