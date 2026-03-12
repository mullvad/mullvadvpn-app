package net.mullvad.mullvadvpn.feature.login.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Serializable data class ApiUnreachableNavKey(val args: ApiUnreachableInfoDialogNavArgs) : NavKey

@Serializable data class ApiUnreachableInfoDialogNavArgs(val action: LoginAction)

@Serializable
enum class LoginAction {
    LOGIN,
    CREATE_ACCOUNT,
}

@Serializable
sealed interface ApiUnreachableInfoDialogResult : NavResult {
    data class Success(val arg: ApiUnreachableInfoDialogNavArgs) : ApiUnreachableInfoDialogResult

    data object Error : ApiUnreachableInfoDialogResult
}
