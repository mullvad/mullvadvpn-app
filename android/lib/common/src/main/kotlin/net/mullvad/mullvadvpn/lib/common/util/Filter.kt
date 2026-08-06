package net.mullvad.mullvadvpn.lib.common.util

import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayListType

fun shouldFilterByDaita(
    isDaitaEnabled: Boolean,
    isWhenNeededMultihopEnabled: Boolean,
    relayListType: RelayListType,
) =
    when (relayListType) {
        RelayListType.Single -> isDaitaEnabled && !isWhenNeededMultihopEnabled
        is RelayListType.Multihop ->
            isDaitaEnabled && relayListType.hopType == RelayHopType.ENTRY
    }

fun shouldFilterByQuic(isQuicEnabled: Boolean, relayListType: RelayListType) =
    when (relayListType) {
        RelayListType.Single -> isQuicEnabled
        is RelayListType.Multihop ->
            isQuicEnabled && relayListType.hopType == RelayHopType.ENTRY
    }

fun shouldFilterByLwo(isLwoEnable: Boolean, relayListType: RelayListType) =
    when (relayListType) {
        RelayListType.Single -> isLwoEnable
        is RelayListType.Multihop ->
            isLwoEnable && relayListType.hopType == RelayHopType.ENTRY
    }
