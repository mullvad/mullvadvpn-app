package net.mullvad.mullvadvpn.feature.addtime.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2

@Parcelize
data class AddTimeNavKey(val visible: Boolean) : NavKey2
