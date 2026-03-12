package net.mullvad.mullvadvpn.feature.login.api

import android.os.Parcelable
import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Parcelize
data class ApiUnreachableNavKey(val args: ApiUnreachableInfoDialogNavArgs) : NavKey2

@Parcelize data class ApiUnreachableInfoDialogNavArgs(val action: LoginAction) : Parcelable

@Parcelize
enum class LoginAction : Parcelable {
    LOGIN,
    CREATE_ACCOUNT,
}

@Parcelize
sealed interface ApiUnreachableInfoDialogResult : NavResult {
    data class Success(val arg: ApiUnreachableInfoDialogNavArgs) : ApiUnreachableInfoDialogResult

    data object Error : ApiUnreachableInfoDialogResult
}
