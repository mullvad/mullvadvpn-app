package net.mullvad.mullvadvpn.feature.login.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable
data class LoginNavKey(val accountNumber: String? = null) : NavKey
