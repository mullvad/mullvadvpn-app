package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

enum class RelayListItemContentType {
    CUSTOM_LIST_HEADER,
    CUSTOM_LIST_ITEM,
    CUSTOM_LIST_ENTRY_ITEM,
    CUSTOM_LIST_FOOTER,
    LOCATION_HEADER,
    LOCATION_ITEM,
    LOCATIONS_EMPTY_TEXT,
    EMPTY_RELAY_LIST,
    RECENT_LIST_ITEM,
    RECENT_LIST_HEADER,
    RECENT_LIST_FOOTER,
    SECTION_DIVIDER,
}

enum class RelayListItemState {
    USED_AS_ENTRY,
    USED_AS_EXIT,
}

sealed interface RelayListItem {
    val key: Any
    val contentType: RelayListItemContentType

    sealed interface SelectableItem : RelayListItem {
        val item: RelayItem
        val hierarchy: Hierarchy
        val isSelected: Boolean
        val expanded: Boolean
        val canExpand: Boolean
        val state: RelayListItemState?
        val itemPosition: Position
    }

    data object CustomListHeader : RelayListItem {
        override val key = "custom_list_header"
        override val contentType = RelayListItemContentType.CUSTOM_LIST_HEADER
    }

    data class CustomListItem(
        override val item: RelayItem.CustomList,
        override val isSelected: Boolean = false,
        override val expanded: Boolean = false,
        override val state: RelayListItemState? = null,
        override val itemPosition: Position = Position.Single,
    ) : SelectableItem {
        override val key = item.id
        override val hierarchy = Hierarchy.Parent
        override val contentType = RelayListItemContentType.CUSTOM_LIST_ITEM
        override val canExpand: Boolean = item.hasChildren
    }

    data class CustomListEntryItem(
        val parentId: CustomListId,
        val parentName: CustomListName,
        override val item: RelayItem.Location,
        override val expanded: Boolean,
        override val hierarchy: Hierarchy,
        override val state: RelayListItemState? = null,
        override val itemPosition: Position,
    ) : SelectableItem {
        override val key = parentId to item.id

        // Can't be displayed as selected
        override val isSelected: Boolean = false
        override val contentType = RelayListItemContentType.CUSTOM_LIST_ENTRY_ITEM
        override val canExpand: Boolean = item.hasChildren
    }

    data class CustomListFooter(val hasCustomList: Boolean) : RelayListItem {
        override val key = "custom_list_footer"
        override val contentType = RelayListItemContentType.CUSTOM_LIST_FOOTER
    }

    data object LocationHeader : RelayListItem {
        override val key = "location_header"
        override val contentType = RelayListItemContentType.LOCATION_HEADER
    }

    data class GeoLocationItem(
        override val item: RelayItem.Location,
        override val isSelected: Boolean = false,
        override val hierarchy: Hierarchy,
        override val expanded: Boolean = false,
        override val state: RelayListItemState? = null,
        override val itemPosition: Position,
    ) : SelectableItem {
        override val key = item.id
        override val contentType = RelayListItemContentType.LOCATION_ITEM
        override val canExpand: Boolean = item.hasChildren
    }

    data object RecentsListHeader : RelayListItem {
        override val key = "recents_list_header"
        override val contentType = RelayListItemContentType.RECENT_LIST_HEADER
    }

    data class RecentListItem(
        override val item: RelayItem,
        override val isSelected: Boolean = false,
        override val expanded: Boolean = false,
        override val state: RelayListItemState? = null,
        override val itemPosition: Position = Position.Single,
    ) : SelectableItem {
        override val key = "recents$item"
        override val hierarchy: Hierarchy = Hierarchy.Parent
        override val contentType = RelayListItemContentType.RECENT_LIST_ITEM
        override val canExpand: Boolean = false
    }

    data object RecentsListFooter : RelayListItem {
        override val key = "recents_list_footer"
        override val contentType = RelayListItemContentType.RECENT_LIST_FOOTER
    }

    data class LocationsEmptyText(val searchTerm: String) : RelayListItem {
        override val key = "locations_empty_text"
        override val contentType = RelayListItemContentType.LOCATIONS_EMPTY_TEXT
    }

    data object EmptyRelayList : RelayListItem {
        override val key = "empty_relay_list"
        override val contentType = RelayListItemContentType.EMPTY_RELAY_LIST
    }

    class SectionDivider : RelayListItem {
        override val key: String = "section_divider_${this.hashCode()}"
        override val contentType = RelayListItemContentType.SECTION_DIVIDER
    }
}

data class CheckableRelayListItem(
    val item: RelayItem.Location,
    val checked: Boolean = false,
    val expanded: Boolean = false,
    val itemPosition: Position = Position.Single,
    val hierarchy: Hierarchy = Hierarchy.Parent,
)
