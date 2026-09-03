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
        is RelayListType.Multihop -> isDaitaEnabled && relayListType.hopType == RelayHopType.ENTRY
    }

fun shouldFilterByQuic(isQuicEnabled: Boolean, relayListType: RelayListType) =
    when (relayListType) {
        RelayListType.Single -> isQuicEnabled
        is RelayListType.Multihop -> isQuicEnabled && relayListType.hopType == RelayHopType.ENTRY
    }

fun shouldFilterByLwo(isLwoEnabled: Boolean, relayListType: RelayListType) =
    when (relayListType) {
        RelayListType.Single -> isLwoEnabled
        is RelayListType.Multihop -> isLwoEnabled && relayListType.hopType == RelayHopType.ENTRY
    }

fun shouldFilterByShadowsocks(
    isShadowsocksEnabled: Boolean,
    isShadowsocksPortOutsideOfStandardRange: Boolean,
    relayListType: RelayListType,
) =
    when (relayListType) {
        RelayListType.Single -> isShadowsocksEnabled && isShadowsocksPortOutsideOfStandardRange
        is RelayListType.Multihop ->
            isShadowsocksEnabled &&
                isShadowsocksPortOutsideOfStandardRange &&
                relayListType.hopType == RelayHopType.ENTRY
    }
