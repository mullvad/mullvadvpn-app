package net.mullvad.mullvadvpn.feature.personalvpn.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class PersonalVpnNavKey(val isModal: Boolean = false) : NavKey2
