package net.mullvad.mullvadvpn.feature.location.impl

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.feature.location.impl.search.expandKey
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItemId

internal fun MutableStateFlow<Set<String>>.onToggleExpandSet(
    item: RelayItemId,
    parent: CustomListId? = null,
    expand: Boolean,
) {
    update {
        val key = item.expandKey(parent)
        if (expand) {
            it + key
        } else {
            it - key
        }
    }
}

internal fun MutableStateFlow<Map<String, Boolean>>.onToggleExpandMap(
    item: RelayItemId,
    parent: CustomListId? = null,
    expand: Boolean,
) {
    update {
        val key = item.expandKey(parent)
        it + (key to expand)
    }
}
