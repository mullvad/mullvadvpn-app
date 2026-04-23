package net.mullvad.mullvadvpn.feature.customvpn.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class CustomTunnelsListNavKey(val isModal: Boolean = false) : NavKey2
