package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem

typealias ModelOwnership = net.mullvad.mullvadvpn.lib.model.Ownership

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
    data class Ownership(val ownership: ModelOwnership) : FilterChip

    data class Provider(val count: Int) : FilterChip

    data object Daita : FilterChip
}

enum class RelayListItemContentType {
    CUSTOM_LIST_HEADER,
    CUSTOM_LIST_ITEM,
    CUSTOM_LIST_ENTRY_ITEM,
    CUSTOM_LIST_FOOTER,
    LOCATION_HEADER,
    LOCATION_ITEM,
    LOCATIONS_EMPTY_TEXT,
}

sealed interface RelayListItem {
    val key: Any
    val contentType: RelayListItemContentType

    data object CustomListHeader : RelayListItem {
        override val key = "custom_list_header"
        override val contentType = RelayListItemContentType.CUSTOM_LIST_HEADER
    }

    sealed interface SelectableItem : RelayListItem {
        val depth: Int
        val isSelected: Boolean
        val expanded: Boolean
    }

    data class CustomListItem(
        val item: RelayItem.CustomList,
        override val isSelected: Boolean = false,
        override val expanded: Boolean = false,
    ) : SelectableItem {
        override val key = item.id
        override val depth: Int = 0
        override val contentType = RelayListItemContentType.CUSTOM_LIST_ITEM
    }

    data class CustomListEntryItem(
        val parentId: CustomListId,
        val parentName: CustomListName,
        val item: RelayItem.Location,
        override val expanded: Boolean,
        override val depth: Int = 0,
    ) : SelectableItem {
        override val key = parentId to item.id

        // Can't be displayed as selected
        override val isSelected: Boolean = false
        override val contentType = RelayListItemContentType.CUSTOM_LIST_ENTRY_ITEM
    }

    data class CustomListFooter(val hasCustomList: Boolean) : RelayListItem {
        override val key = "custom_list_footer"
        override val contentType = RelayListItemContentType.CUSTOM_LIST_FOOTER
    }

    data object LocationHeader : RelayListItem {
        override val key: Any = "location_header"
        override val contentType = RelayListItemContentType.LOCATION_HEADER
    }

    data class GeoLocationItem(
        val item: RelayItem.Location,
        override val isSelected: Boolean = false,
        override val depth: Int = 0,
        override val expanded: Boolean = false,
    ) : SelectableItem {
        override val key = item.id
        override val contentType = RelayListItemContentType.LOCATION_ITEM
    }

    data class LocationsEmptyText(val searchTerm: String) : RelayListItem {
        override val key: Any = "locations_empty_text"
        override val contentType = RelayListItemContentType.LOCATIONS_EMPTY_TEXT
    }
}
