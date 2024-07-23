package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItem

sealed interface SelectLocationUiState {

    data object Loading : SelectLocationUiState

    data class Content(
        val searchTerm: String,
        val filterChips: List<FilterChip>,
        val relayListItems: List<RelayListItem>,
        val customLists: List<RelayItem.CustomList>,
    ) : SelectLocationUiState
}

sealed interface FilterChip {
    data class Ownership(val ownership: net.mullvad.mullvadvpn.lib.model.Ownership) : FilterChip

    data class Provider(val count: Int) : FilterChip
}

sealed interface RelayListItem {
    val key: Any

    data object CustomListHeader : RelayListItem {
        override val key = "custom_list_header"
    }

    sealed interface SelectableItem : RelayListItem {
        val depth: Int
        val isSelected: Boolean
        val expanded: Boolean
    }

    data class CustomListItem(
        val item: RelayItem.CustomList,
        override val isSelected: Boolean,
        override val expanded: Boolean,
    ) : SelectableItem {
        override val key = item.id
        override val depth: Int = 0
    }

    data class CustomListEntryItem(
        val parentId: CustomListId,
        val item: RelayItem.Location,
        override val expanded: Boolean,
        override val depth: Int = 0
    ) : SelectableItem {
        override val key = parentId to item.id

        // Can't be displayed as selected
        override val isSelected: Boolean = false
    }

    data class GeoLocationItem(
        val item: RelayItem.Location,
        override val isSelected: Boolean,
        override val depth: Int,
        override val expanded: Boolean,
    ) : SelectableItem {
        override val key = item.id
    }

    data class CustomListFooter(val hasCustomList: Boolean) : RelayListItem {
        override val key = "custom_list_footer"
    }

    data object LocationHeader : RelayListItem {
        override val key: Any = "location_header"
    }

    data class LocationsEmptyText(val searchTerm: String) : RelayListItem {
        override val key: Any = "locations_empty_text"
    }
}
