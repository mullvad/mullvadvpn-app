package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class CreateCustomListResult : Parcelable {
    @Parcelize data class Ok(val id: String) : CreateCustomListResult()

    @Parcelize data class Error(val error: CustomListsError) : CreateCustomListResult()
}
