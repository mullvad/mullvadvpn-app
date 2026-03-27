package net.mullvad.mullvadvpn.feature.login.api

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize data class ApiUnreachableNavKey(val action: LoginAction) : NavKey2

@Parcelize
enum class LoginAction : Parcelable {
    LOGIN,
    CREATE_ACCOUNT,
}

@Parcelize
sealed interface ApiUnreachableInfoDialogResult : NavResult {
    data class Success(val arg: ApiUnreachableNavKey) : ApiUnreachableInfoDialogResult

    data object Error : ApiUnreachableInfoDialogResult
}
