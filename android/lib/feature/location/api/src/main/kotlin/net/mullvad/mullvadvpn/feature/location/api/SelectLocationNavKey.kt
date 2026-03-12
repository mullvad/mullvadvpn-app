package net.mullvad.mullvadvpn.feature.location.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Serializable object SelectLocationNavKey : NavKey

@Serializable data class SelectLocationNavResult(val connect: Boolean) : NavResult

