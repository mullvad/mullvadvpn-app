package net.mullvad.mullvadvpn.feature.appinfo.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class ChangelogNavKey(val isModal: Boolean = false) : NavKey2
