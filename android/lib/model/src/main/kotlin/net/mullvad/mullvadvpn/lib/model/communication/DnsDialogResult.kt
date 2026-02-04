package net.mullvad.mullvadvpn.lib.model.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed interface DnsDialogResult : Parcelable {
    @Parcelize data class Success(val isDnsListEmpty: Boolean) : DnsDialogResult

    @Parcelize data object Error : DnsDialogResult
}
