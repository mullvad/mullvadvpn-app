package net.mullvad.mullvadvpn.compose.component

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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadGreen
import net.mullvad.mullvadvpn.compose.theme.MullvadRed
import net.mullvad.mullvadvpn.relaylist.Relay
import net.mullvad.mullvadvpn.relaylist.RelayCity
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayItemType

@Composable
@Preview
fun PreviewRelayLocation() {
    AppTheme {
        Column(Modifier.background(color = MullvadDarkBlue)) {
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
                    country = relayCountry,
                    relays = emptyList()
                )
            val relay =
                Relay(
                    name = "Relay",
                    city = relayCity,
                    active = false,
                )
            val relayCountryAndCity =
                RelayCountry(
                    name = "Relay Country",
                    code = "RC",
                    expanded = true,
                    cities =
                        listOf(
                            RelayCity(
                                country = relayCountry,
                                "Relay City",
                                code = "RCI",
                                expanded = false,
                                emptyList()
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
                                country = relayCountry,
                                "Relay City",
                                code = "RCI",
                                expanded = true,
                                relays =
                                    listOf(
                                        Relay(city = relayCity, name = "Relay Item", active = false)
                                    )
                            )
                        )
                )
            // Relay only country
            RelayLocation(relayCountry)
            // Relay country and city
            RelayLocation(relayCountryAndCity)
            // Relay country, city and relay
            RelayLocation(fullRelayList)
            // Relay only city not expanded
            RelayLocation(relayCity)
            // Relay only not active
            RelayLocation(relay)
            // Relay only active
            RelayLocation(relay = relay, selectedItem = relay)
        }
    }
}

@Composable
fun RelayLocation(
    relay: RelayItem,
    modifier: Modifier = Modifier,
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
    val expanded = rememberSaveable { mutableStateOf(relay.expanded) }
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
                    .padding(vertical = Dimens.listItemDivider)
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
                    .then(
                        if (relay.active) {
                            Modifier.clickable { onSelectRelay(relay) }
                        } else {
                            Modifier
                        }
                    )
        ) {
            Box(
                modifier = Modifier.align(Alignment.CenterVertically).padding(start = startPadding)
            ) {
                Box(
                    modifier =
                        Modifier.align(Alignment.CenterStart)
                            .size(16.dp)
                            .background(
                                color =
                                    when {
                                        selected -> Color.Transparent
                                        relay.active -> MullvadGreen
                                        else -> MullvadRed
                                    },
                                shape = RoundedCornerShape(16.dp)
                            )
                )
                Image(
                    painter = painterResource(id = R.drawable.icon_tick),
                    modifier =
                        Modifier.align(Alignment.CenterStart)
                            .alpha(
                                if (selected) {
                                    1f
                                } else {
                                    0f
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
                                1f
                            } else {
                                0.5f
                            }
                        )
                        .padding(horizontal = 8.dp, vertical = 14.dp)
            )
            if (relay.hasChildren) {
                Image(
                    painter = painterResource(id = R.drawable.icon_chevron),
                    contentDescription = null,
                    modifier =
                        Modifier.fillMaxHeight()
                            .clickable { expanded.value = !expanded.value }
                            .padding(horizontal = 16.dp)
                            .align(Alignment.CenterVertically)
                            .rotate(
                                if (expanded.value) {
                                    270f
                                } else {
                                    90f
                                }
                            )
                )
            }
        }
        if (expanded.value) {
            when (relay) {
                is RelayCountry -> {
                    relay.cities.forEach { relayCity ->
                        RelayLocation(
                            relay = relayCity,
                            selectedItem = selectedItem,
                            onSelectRelay = onSelectRelay,
                            modifier = Modifier.animateContentSize()
                        )
                    }
                }
                is RelayCity -> {
                    relay.relays.forEach { relay ->
                        RelayLocation(
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
