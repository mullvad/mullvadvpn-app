package net.mullvad.mullvadvpn.feature.location.impl

import net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayMetadata
import net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayMetadataMap
import net.mullvad.mullvadvpn.lib.common.util.relaylist.merge
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.SearchMatch

internal fun RelayMetadataMap.withHighlights(
    highlights: Map<RelayItemId, SearchMatch>
): RelayMetadataMap {
    val highlightsMetadata = highlights.mapValues { (_, match) ->
        RelayMetadata(titleHighlights = match.matchRange)
    }
    return merge(highlightsMetadata)
}

internal fun RelayMetadataMap.withConnectedRelay(connectedRelay: RelayItemId?): RelayMetadataMap {
    if (connectedRelay == null) return this
    return merge(mapOf(connectedRelay to RelayMetadata(connectedRelay = true)))
}
