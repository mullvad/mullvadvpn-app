package net.mullvad.mullvadvpn.feature.location.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.RelayListType

@Parcelize data class SearchLocationNavKey(val relayListType: RelayListType) : NavKey2

@Parcelize data class SearchLocationNavResult(val relayListType: RelayListType) : NavResult
