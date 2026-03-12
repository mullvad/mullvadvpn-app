package net.mullvad.mullvadvpn.feature.daita.api

import android.os.Parcelable
import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Parcelize
object DaitaDirectOnlyConfirmationNavKey : NavKey2

@Parcelize
data class DaitaDirectOnlyConfirmationNavResult(val confirmed: Boolean) : NavResult

@Parcelize data class DaitaNavArgs(val isModal: Boolean = false) : Parcelable
