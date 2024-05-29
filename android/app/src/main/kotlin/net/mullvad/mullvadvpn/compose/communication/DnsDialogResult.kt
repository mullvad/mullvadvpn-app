package net.mullvad.mullvadvpn.compose.communication

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

interface DnsDialogResult : Parcelable {
    @Parcelize data object Success : DnsDialogResult

    @Parcelize data object Error : DnsDialogResult

    @Parcelize data object Cancel : DnsDialogResult
}
