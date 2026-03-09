package net.mullvad.mullvadvpn.feature.customlist.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize object DiscardCustomListChangesNavKey : NavKey2

@Parcelize object DiscardCustomListChangesConfirmedNavResult : NavResult
