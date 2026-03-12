package net.mullvad.mullvadvpn.feature.daita.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2

@Parcelize
data class DaitaNavKey(val isModal: Boolean = false) : NavKey2
