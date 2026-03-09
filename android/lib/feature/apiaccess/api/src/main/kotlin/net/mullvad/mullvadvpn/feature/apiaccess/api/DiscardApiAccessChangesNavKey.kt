package net.mullvad.mullvadvpn.feature.apiaccess.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize object DiscardApiAccessChangesNavKey : NavKey2

@Parcelize data object DiscardApiAccessChangesConfirmedNavResult : NavResult
