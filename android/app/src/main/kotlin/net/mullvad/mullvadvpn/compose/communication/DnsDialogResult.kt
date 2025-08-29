package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

interface DnsDialogResult : Parcelable {
    @Parcelize data class Success(val isDnsListEmpty: Boolean) : DnsDialogResult

    @Parcelize data object Error : DnsDialogResult
}
