package net.mullvad.mullvadvpn.compose.state

import arrow.core.NonEmptyList
import net.mullvad.mullvadvpn.lib.common.util.getFirstInstanceOrNull
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
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

        fun fromCustomProxy(
            name: ApiAccessMethodName,
            customProxy: ApiAccessMethodType.CustomProxy
        ) =
            when (customProxy) {
                is ApiAccessMethodType.CustomProxy.Shadowsocks -> {
                    EditApiAccessFormData(
                        name = name.value,
                        serverIp = customProxy.ip,
                        port = customProxy.port.toString(),
                        password = customProxy.password ?: "",
                        cipher = customProxy.cipher,
                        username = "",
                    )
                }
                is ApiAccessMethodType.CustomProxy.Socks5Remote ->
                    EditApiAccessFormData(
                        name = name.value,
                        serverIp = customProxy.ip,
                        port = customProxy.port.toString(),
                        enableAuthentication = customProxy.auth != null,
                        username = customProxy.auth?.username ?: "",
                        password = customProxy.auth?.password ?: ""
                    )
            }
    }
}

enum class ApiAccessMethodTypes {
    SOCKS5_REMOTE,
    SHADOWSOCKS;

    companion object {
        fun default(): ApiAccessMethodTypes = SHADOWSOCKS
    }
}
