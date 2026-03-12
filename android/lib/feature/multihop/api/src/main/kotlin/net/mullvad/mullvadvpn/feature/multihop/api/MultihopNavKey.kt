package net.mullvad.mullvadvpn.feature.multihop.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2

@Parcelize
data class MultihopNavKey(val isModal: Boolean = false) : NavKey2
