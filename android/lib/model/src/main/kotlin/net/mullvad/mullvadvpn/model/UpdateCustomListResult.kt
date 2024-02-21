package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class UpdateCustomListResult : Parcelable {
    @Parcelize data object Ok : UpdateCustomListResult()

    @Parcelize data class Error(val error: CustomListsError) : UpdateCustomListResult()
}
