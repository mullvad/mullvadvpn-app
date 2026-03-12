package net.mullvad.mullvadvpn.feature.splittunneling.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2

@Parcelize
data class SplitTunnelingNavKey(val isModal: Boolean = false) : NavKey2
