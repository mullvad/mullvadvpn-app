package net.mullvad.mullvadvpn.feature.lansharing.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class LocalNetworkSharingNavKey(val isModal: Boolean = false) : NavKey2
