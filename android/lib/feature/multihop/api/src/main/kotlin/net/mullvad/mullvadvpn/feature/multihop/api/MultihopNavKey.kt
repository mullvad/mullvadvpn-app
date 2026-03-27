package net.mullvad.mullvadvpn.feature.multihop.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class MultihopNavKey(val isModal: Boolean = false) : NavKey2
