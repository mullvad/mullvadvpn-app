package net.mullvad.mullvadvpn.compose.state

import arrow.core.NonEmptyList
import net.mullvad.mullvadvpn.lib.common.util.getFirstInstanceOrNull
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState

sealed interface EditApiAccessMethodUiState {
    val editMode: Boolean

    data class Loading(override val editMode: Boolean) : EditApiAccessMethodUiState

    data class Content(
        override val editMode: Boolean,
        val formData: EditApiAccessFormData,
        val testMethodState: TestApiAccessMethodState?
    ) : EditApiAccessMethodUiState
}

data class EditApiAccessFormData(
    val name: String,
    val nameError: InvalidDataError.NameError? = null,
    val apiAccessMethodTypes: ApiAccessMethodTypes = ApiAccessMethodTypes.default(),
    val serverIp: String,
    val serverIpError: InvalidDataError.ServerIpError? = null,
    val port: String,
    val portError: InvalidDataError.PortError? = null,
    val enableAuthentication: Boolean = false,
    val username: String,
    val usernameError: InvalidDataError.UserNameError? = null,
    val password: String,
    val passwordError: InvalidDataError.PasswordError? = null,
    val cipher: Cipher = Cipher.first()
) {
    fun updateWithErrors(errors: NonEmptyList<InvalidDataError>): EditApiAccessFormData =
        copy(
            nameError = errors.getFirstInstanceOrNull(),
            serverIpError = errors.getFirstInstanceOrNull(),
            portError = errors.getFirstInstanceOrNull(),
            usernameError = errors.getFirstInstanceOrNull(),
            passwordError = errors.getFirstInstanceOrNull()
        )

    companion object {
        fun empty() =
            EditApiAccessFormData(name = "", password = "", port = "", serverIp = "", username = "")
    }
}

enum class ApiAccessMethodTypes {
    SOCKS5_REMOTE,
    SHADOWSOCKS;

    companion object {
        fun default(): ApiAccessMethodTypes = SHADOWSOCKS
    }
}
