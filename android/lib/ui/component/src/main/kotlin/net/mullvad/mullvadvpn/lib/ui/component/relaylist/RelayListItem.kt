package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import android.content.Context
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.resource.R

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
    SECTION_DIVIDER,
}

enum class RelayListItemState {
    USED_AS_ENTRY,
    USED_AS_EXIT,
}

sealed interface RelayListItem {
    val key: Any
    val contentType: RelayListItemContentType

    data object CustomListHeader : RelayListItem {
        override val key = "custom_list_header"
        override val contentType = RelayListItemContentType.CUSTOM_LIST_HEADER
    }

    data object RecentsListHeader : RelayListItem {
        override val key = "recents_list_header"
        override val contentType = RelayListItemContentType.RECENT_LIST_HEADER
    }

    sealed interface SelectableItem : RelayListItem {
        val hop: Hop
        val depth: Int
        val isSelected: Boolean
        val expanded: Boolean
        val canExpand: Boolean
        val state: RelayListItemState?
        val itemPosition: ItemPosition
    }

    data class CustomListItem(
        override val hop: Hop.Single<RelayItem.CustomList>,
        override val isSelected: Boolean = false,
        override val expanded: Boolean = false,
        override val state: RelayListItemState? = null,
        override val itemPosition: ItemPosition = ItemPosition.Single,
    ) : SelectableItem {
        val item = hop.item
        override val key = item.id
        override val depth: Int = 0
        override val contentType = RelayListItemContentType.CUSTOM_LIST_ITEM
        override val canExpand: Boolean = item.hasChildren
    }

    data class CustomListEntryItem(
        val parentId: CustomListId,
        val parentName: CustomListName,
        override val hop: Hop.Single<RelayItem.Location>,
        override val expanded: Boolean,
        override val depth: Int = 0,
        override val state: RelayListItemState? = null,
        override val itemPosition: ItemPosition,
    ) : SelectableItem {
        val item = hop.item
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
        override val hop: Hop.Single<RelayItem.Location>,
        override val isSelected: Boolean = false,
        override val depth: Int = 0,
        override val expanded: Boolean = false,
        override val state: RelayListItemState? = null,
        override val itemPosition: ItemPosition,
    ) : SelectableItem {
        val item = hop.item
        override val key = item.id
        override val contentType = RelayListItemContentType.LOCATION_ITEM
        override val canExpand: Boolean = item.hasChildren
    }

    data class RecentListItem(
        override val hop: Hop,
        override val isSelected: Boolean = false,
        override val expanded: Boolean = false,
        override val state: RelayListItemState? = null,
        override val itemPosition: ItemPosition = ItemPosition.Single,
    ) : SelectableItem {
        override val key = "recents$hop"
        override val depth: Int = 0
        override val contentType = RelayListItemContentType.RECENT_LIST_ITEM
        override val canExpand: Boolean = false
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
    val depth: Int = 0,
    val checked: Boolean = false,
    val expanded: Boolean = false,
    val itemPosition: ItemPosition = ItemPosition.Single,
)

sealed interface ItemPosition {
    data object Top : ItemPosition

    data object Middle : ItemPosition

    data object Bottom : ItemPosition

    data object Single : ItemPosition

    fun roundTop(): Boolean =
        when (this) {
            is Single,
            Top -> true
            else -> false
        }

    fun roundBottom(): Boolean =
        when (this) {
            is Single,
            Bottom -> true
            else -> false
        }
}

fun Hop.displayName(context: Context): String =
    when (this) {
        is Hop.Multi -> context.getString(R.string.x_via_x, exit.name, entry.name)
        is Hop.Single<*> -> item.name
    }
