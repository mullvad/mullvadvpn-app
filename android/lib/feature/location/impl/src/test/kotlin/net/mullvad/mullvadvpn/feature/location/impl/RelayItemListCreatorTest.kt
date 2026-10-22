package net.mullvad.mullvadvpn.feature.location.impl

import kotlin.test.assertEquals
import kotlin.test.assertTrue
import net.mullvad.mullvadvpn.feature.location.impl.data.CUSTOM_LIST_AA
import net.mullvad.mullvadvpn.feature.location.impl.data.RELAY_COUNTRY_BELGIUM
import net.mullvad.mullvadvpn.feature.location.impl.data.RELAY_COUNTRY_SWEDEN
import net.mullvad.mullvadvpn.feature.location.impl.data.RELAY_ID_BE_BRU_WG_101
import net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayMetadata
import net.mullvad.mullvadvpn.lib.model.Constraint
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
                RELAY_ID_BE_BRU_WG_101 to
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
                relayCountries = listOf(RELAY_COUNTRY_BELGIUM),
                customLists = listOf(CUSTOM_LIST_AA),
                recents = null,
                selectedItem = RelayItemSelection.Single(exitLocation = Constraint.Any),
                selectedByThisEntryExitList = null,
                selectedByOtherEntryExitList = null,
                expandedItems =
                    setOf(
                        RELAY_ID_BE_BRU_WG_101.country.code,
                        RELAY_ID_BE_BRU_WG_101.city.code,
                        CUSTOM_LIST_AA.id.value,
                    ),
            )

        assertEquals(
            1,
            relayListItems.filterIsInstance<RelayListItem.CustomListEntryItem>().count {
                it.item.id == RELAY_ID_BE_BRU_WG_101 && it.connected
            },
        )
        assertEquals(
            1,
            relayListItems.filterIsInstance<RelayListItem.GeoLocationItem>().count {
                it.item.id == RELAY_ID_BE_BRU_WG_101 && it.connected
            },
        )
        assertEquals(
            2,
            relayListItems.filterIsInstance<RelayListItem.SelectableItem>().count { it.connected },
        )
    }

    @Test
    fun `when metadata does not contain any connected server no server should be connected`() {
        val relayListItems =
            relayListItems(
                relayListType = RelayListType.Single,
                relayMetadata = emptyMap(),
                relayCountries = listOf(RELAY_COUNTRY_BELGIUM),
                customLists = listOf(CUSTOM_LIST_AA),
                recents = null,
                selectedItem = RelayItemSelection.Single(exitLocation = Constraint.Any),
                selectedByThisEntryExitList = null,
                selectedByOtherEntryExitList = null,
                expandedItems =
                    setOf(
                        RELAY_ID_BE_BRU_WG_101.country.code,
                        RELAY_ID_BE_BRU_WG_101.city.code,
                        CUSTOM_LIST_AA.id.value,
                    ),
            )

        assertTrue(
            relayListItems.filterIsInstance<RelayListItem.SelectableItem>().none { it.connected }
        )
    }

    @Test
    fun `when metadata contains connected server that is not in the relay items, then no server should be connected`() {
        val metadataMap =
            mapOf<RelayItemId, RelayMetadata>(
                RELAY_ID_BE_BRU_WG_101 to
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
                relayCountries = listOf(RELAY_COUNTRY_SWEDEN),
                customLists = emptyList(),
                recents = null,
                selectedItem = RelayItemSelection.Single(exitLocation = Constraint.Any),
                selectedByThisEntryExitList = null,
                selectedByOtherEntryExitList = null,
                expandedItems = emptySet(),
            )

        assertTrue(
            relayListItems.filterIsInstance<RelayListItem.SelectableItem>().none { it.connected }
        )
    }
}
