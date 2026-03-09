package net.mullvad.mullvadvpn.feature.login.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class LoginNavKey(val accountNumber: String? = null) : NavKey2
