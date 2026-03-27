package net.mullvad.mullvadvpn.feature.daita.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class DaitaNavKey(val isModal: Boolean = false) : NavKey2
