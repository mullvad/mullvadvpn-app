package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ChevronView
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.AlphaInvisible
import net.mullvad.mullvadvpn.compose.theme.AlphaVisible
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.relaylist.Relay
import net.mullvad.mullvadvpn.relaylist.RelayCity
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayItemType

@Composable
@Preview
private fun PreviewRelayLocationCell() {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.background)) {
            val relayCountry =
                RelayCountry(
                    name = "Relay only country",
                    code = "ROC",
                    expanded = false,
                    cities = emptyList()
                )
            val relayCity =
                RelayCity(
                    name = "Relay only city",
                    code = "RCC",
                    expanded = false,
                    location = LocationConstraint.City("ROC", "RCC"),
                    relays = emptyList()
                )
            val relay =
                Relay(
                    name = "Relay",
                    active = false,
                    location = LocationConstraint.Hostname("ROC", "RCC", "HNAME"),
                    locationName = "Relay only city (Relay)"
                )
            val relayCountryAndCity =
                RelayCountry(
                    name = "Relay Country",
                    code = "RC",
                    expanded = true,
                    cities =
                        listOf(
                            RelayCity(
                                name = "Relay City",
                                code = "RCI",
                                expanded = false,
                                location = LocationConstraint.City("RC", "RCI"),
                                relays = emptyList()
                            )
                        )
                )
            val fullRelayList =
                RelayCountry(
                    name = "Relay Country",
                    code = "RC",
                    expanded = true,
                    cities =
                        listOf(
                            RelayCity(
                                name = "Relay City",
                                code = "RCI",
                                expanded = true,
                                location = LocationConstraint.City("RC", "RCI"),
                                relays =
                                    listOf(
                                        Relay(
                                            name = "Relay Item",
                                            active = false,
                                            location =
                                                LocationConstraint.Hostname("RC", "RCI", "HNAME"),
                                            locationName = "Relay city (Relay Item)"
                                        )
                                    )
                            )
                        )
                )
            // Relay only country
            RelayLocationCell(relayCountry)
            // Relay country and city
            RelayLocationCell(relayCountryAndCity)
            // Relay country, city and relay
            RelayLocationCell(fullRelayList)
            // Relay only city not expanded
            RelayLocationCell(relayCity)
            // Relay only not active
            RelayLocationCell(relay)
            // Relay only active
            RelayLocationCell(relay = relay, selectedItem = relay)
        }
    }
}

@Composable
fun RelayLocationCell(
    relay: RelayItem,
    modifier: Modifier = Modifier,
    activeColor: Color = MaterialTheme.colorScheme.surface,
    inactiveColor: Color = MaterialTheme.colorScheme.error,
    selectedItem: RelayItem? = null,
    onSelectRelay: (item: RelayItem) -> Unit = {}
) {
    val startPadding =
        when (relay.type) {
            RelayItemType.Country -> Dimens.countryRowPadding
            RelayItemType.City -> Dimens.cityRowPadding
            RelayItemType.Relay -> Dimens.relayRowPadding
        }
    val selected = selectedItem == relay
    val expanded = remember(key1 = relay.expanded.toString()) { mutableStateOf(relay.expanded) }
    val backgroundColor =
        when {
            selected -> MaterialTheme.colorScheme.inversePrimary
            relay.type == RelayItemType.Country -> MaterialTheme.colorScheme.primary
            relay.type == RelayItemType.City -> MaterialTheme.colorScheme.primaryContainer
            relay.type == RelayItemType.Relay -> MaterialTheme.colorScheme.secondaryContainer
            else -> MaterialTheme.colorScheme.primary
        }
    Column(
        modifier =
            modifier.then(
                Modifier.fillMaxWidth()
                    .padding(top = Dimens.listItemDivider)
                    .wrapContentHeight()
                    .fillMaxWidth()
            )
    ) {
        Row(
            modifier =
                Modifier.align(Alignment.Start)
                    .wrapContentHeight()
                    .height(IntrinsicSize.Min)
                    .fillMaxWidth()
                    .background(backgroundColor)
        ) {
            Row(
                modifier =
                    Modifier.weight(1f)
                        .then(
                            if (relay.active) {
                                Modifier.clickable { onSelectRelay(relay) }
                            } else {
                                Modifier
                            }
                        )
            ) {
                Box(
                    modifier =
                        Modifier.align(Alignment.CenterVertically).padding(start = startPadding)
                ) {
                    Box(
                        modifier =
                            Modifier.align(Alignment.CenterStart)
                                .size(Dimens.relayCircleSize)
                                .background(
                                    color =
                                        when {
                                            selected -> Color.Transparent
                                            relay.active -> activeColor
                                            else -> inactiveColor
                                        },
                                    shape = CircleShape
                                )
                    )
                    Image(
                        painter = painterResource(id = R.drawable.icon_tick),
                        modifier =
                            Modifier.align(Alignment.CenterStart)
                                .alpha(
                                    if (selected) {
                                        AlphaVisible
                                    } else {
                                        AlphaInvisible
                                    }
                                ),
                        contentDescription = null
                    )
                }
                Text(
                    text = relay.name,
                    color = MaterialTheme.colorScheme.onPrimary,
                    modifier =
                        Modifier.weight(1f)
                            .align(Alignment.CenterVertically)
                            .alpha(
                                if (relay.active) {
                                    AlphaVisible
                                } else {
                                    AlphaInactive
                                }
                            )
                            .padding(
                                horizontal = Dimens.smallPadding,
                                vertical = Dimens.mediumPadding
                            )
                )
            }
            if (relay.hasChildren) {
                ChevronView(
                    isExpanded = expanded.value,
                    modifier =
                        Modifier.fillMaxHeight()
                            .clickable { expanded.value = !expanded.value }
                            .padding(horizontal = Dimens.mediumPadding)
                            .align(Alignment.CenterVertically)
                )
            }
        }
        if (expanded.value) {
            when (relay) {
                is RelayCountry -> {
                    relay.cities.forEach { relayCity ->
                        RelayLocationCell(
                            relay = relayCity,
                            selectedItem = selectedItem,
                            onSelectRelay = onSelectRelay,
                            modifier = Modifier.animateContentSize()
                        )
                    }
                }
                is RelayCity -> {
                    relay.relays.forEach { relay ->
                        RelayLocationCell(
                            relay = relay,
                            selectedItem = selectedItem,
                            onSelectRelay = onSelectRelay,
                            modifier = Modifier.animateContentSize()
                        )
                    }
                }
            }
        }
    }
}
