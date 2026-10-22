package net.mullvad.mullvadvpn.feature.location.impl

import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import net.mullvad.mullvadvpn.feature.location.impl.search.relayListItems
import net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayMetadata
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.DomainCustomList
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import org.junit.jupiter.api.Test

class RelayItemListCreatorTest {

    @Test
    fun `when metadata contains connected server, then connected should be true for only that server`() {
        val metadataMap =
            mapOf<RelayItemId, RelayMetadata>(
                DUMMY_RELAY_ITEM_ID_CONNECTED to
                    RelayMetadata(
                        needsOtherEntry = false,
                        titleHighlights = null,
                        connectedRelay = true,
                    )
            )
        val relayListItems =
            relayListItems(
                relayListType = RelayListType.Single,
                relayMetadata = metadataMap,
                relayCountries = DUMMY_RELAY_ITEMS,
                customLists = listOf(DUMMY_CUSTOM_LIST),
                recents = null,
                selectedItem = RelayItemSelection.Single(exitLocation = Constraint.Any),
                selectedByThisEntryExitList = null,
                selectedByOtherEntryExitList = null,
                expandedItems = setOf("be", "bru", "100"),
            )

        assertEquals(
            1,
            relayListItems.filterIsInstance<RelayListItem.CustomListEntryItem>().count {
                it.item.id == DUMMY_RELAY_ITEM_ID_CONNECTED && it.connected
            },
        )
        assertEquals(
            1,
            relayListItems.filterIsInstance<RelayListItem.GeoLocationItem>().count {
                it.item.id == DUMMY_RELAY_ITEM_ID_CONNECTED && it.connected
            },
        )
        assertEquals(
            2,
            relayListItems.filterIsInstance<RelayListItem.SelectableItem>().count {
                it.connected
            },
        )
    }

    @Test
    fun `when metadata does not contain any connected server no server should be connected`() {
        val relayListItems =
            relayListItems(
                relayListType = RelayListType.Single,
                relayMetadata = emptyMap(),
                relayCountries = DUMMY_RELAY_ITEMS,
                customLists = listOf(DUMMY_CUSTOM_LIST),
                recents = null,
                selectedItem = RelayItemSelection.Single(exitLocation = Constraint.Any),
                selectedByThisEntryExitList = null,
                selectedByOtherEntryExitList = null,
                expandedItems = setOf("be", "bru"),
            )

        assertTrue(
            relayListItems.filterIsInstance<RelayListItem.SelectableItem>().none { it.connected }
        )
    }

    @Test
    fun `when metadata contains connected server that is not in the relay items, then no server should be connected`() {
        val metadataMap =
            mapOf<RelayItemId, RelayMetadata>(
                DUMMY_RELAY_ITEM_ID_OTHER to
                    RelayMetadata(
                        needsOtherEntry = false,
                        titleHighlights = null,
                        connectedRelay = true,
                    )
            )
        val relayListItems =
            relayListItems(
                relayListType = RelayListType.Single,
                relayMetadata = metadataMap,
                relayCountries =
                    listOf(
                        RelayItem.Location.Country(
                            id = GeoLocationId.Country("be"),
                            name = "Belgium - Brussels",
                            cities = emptyList(),
                        )
                    ),
                customLists = emptyList(),
                recents = null,
                selectedItem = RelayItemSelection.Single(exitLocation = Constraint.Any),
                selectedByThisEntryExitList = null,
                selectedByOtherEntryExitList = null,
                expandedItems = setOf("be", "bru"),
            )

        assertTrue(
            relayListItems.filterIsInstance<RelayListItem.SelectableItem>().none { it.connected }
        )
    }

    companion object {
        private val DUMMY_RELAY_ITEM_ID_CONNECTED =
            GeoLocationId.Hostname(
                code = "be-bru-wg-101",
                city =
                    GeoLocationId.City(
                        code = "bru",
                        country = GeoLocationId.Country(code = "be"),
                    ),
            )

        private val DUMMY_RELAY_ITEM_ID_OTHER =
            GeoLocationId.Hostname(
                code = "be-bru-wg-102",
                city =
                    GeoLocationId.City(
                        code = "bru",
                        country = GeoLocationId.Country(code = "be"),
                    ),
            )

        private val DUMMY_RELAY_ITEMS =
            listOf(
                RelayItem.Location.Country(
                    id = GeoLocationId.Country("be"),
                    name = "Belgium - Brussels",
                    cities =
                        listOf(
                            RelayItem.Location.City(
                                id =
                                    GeoLocationId.City(
                                        code = "bru",
                                        country = GeoLocationId.Country(code = "be"),
                                    ),
                                name = "Brussels",
                                latLong = mockk(relaxed = true),
                                countryName = "Belgium",
                                relays =
                                    listOf(
                                        RelayItem.Location.Relay(
                                            id = DUMMY_RELAY_ITEM_ID_CONNECTED,
                                            active = true,
                                            latLong = mockk(relaxed = true),
                                            provider = ProviderId("mullvad"),
                                            ownership = Ownership.MullvadOwned,
                                            daita = false,
                                            quic = null,
                                            lwo = false,
                                            cityName = "Brussels",
                                            countryName = "Belgium",
                                        ),
                                        RelayItem.Location.Relay(
                                            id = DUMMY_RELAY_ITEM_ID_OTHER,
                                            active = true,
                                            latLong = mockk(relaxed = true),
                                            provider = ProviderId("mullvad"),
                                            ownership = Ownership.MullvadOwned,
                                            daita = false,
                                            quic = null,
                                            lwo = false,
                                            cityName = "Brussels",
                                            countryName = "Belgium",
                                        ),
                                    ),
                            )
                        ),
                )
            )

        private val DUMMY_CUSTOM_LIST =
            RelayItem.CustomList(
                customList =
                    DomainCustomList(
                        id = CustomListId("100"),
                        name = CustomListName.fromString("Custom List 1"),
                        locations =
                            listOf(
                                GeoLocationId.Hostname(
                                    code = DUMMY_RELAY_ITEM_ID_CONNECTED.code,
                                    city =
                                        GeoLocationId.City(
                                            code = DUMMY_RELAY_ITEM_ID_CONNECTED.city.code,
                                            country =
                                                GeoLocationId.Country(
                                                    code =
                                                        DUMMY_RELAY_ITEM_ID_CONNECTED.city.country
                                                            .code
                                                ),
                                        ),
                                )
                            ),
                    ),
                locations =
                    listOf(
                        RelayItem.Location.Relay(
                            id = DUMMY_RELAY_ITEM_ID_CONNECTED,
                            active = true,
                            latLong = mockk(relaxed = true),
                            provider = ProviderId("mullvad"),
                            ownership = Ownership.MullvadOwned,
                            daita = false,
                            quic = null,
                            lwo = false,
                            cityName = "Brussels",
                            countryName = "Belgium",
                        )
                    ),
            )
    }
}
