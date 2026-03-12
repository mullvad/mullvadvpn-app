package net.mullvad.mullvadvpn.feature.location.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Parcelize
object SelectLocationNavKey : NavKey2

@Parcelize data class SelectLocationNavResult(val connect: Boolean) : NavResult

