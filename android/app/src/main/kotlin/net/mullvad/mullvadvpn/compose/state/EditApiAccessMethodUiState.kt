package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState

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
    val name: FormInputField<InvalidDataError.NameError>,
    val apiAccessMethodTypes: ApiAccessMethodTypes,
    val ip: FormInputField<InvalidDataError.ServerIpError>,
    val remotePort: FormInputField<InvalidDataError.RemotePortError>,
    val enableAuthentication: Boolean,
    val username: FormInputField<InvalidDataError.UserNameError>,
    val password: FormInputField<InvalidDataError.PasswordError>,
    val cipher: Cipher
) {

    fun updateName(name: String) = copy(name = FormInputField(name))

    fun updateServerIp(ip: String) = copy(ip = FormInputField(ip))

    fun updateRemotePort(port: String) = copy(remotePort = FormInputField(port))

    fun updatePassword(password: String) = copy(password = FormInputField(password))

    fun updateCipher(cipher: Cipher) = copy(cipher = cipher)

    fun updateAuthenticationEnabled(enableAuthentication: Boolean) =
        copy(enableAuthentication = enableAuthentication)

    fun updateUsername(username: String) = copy(username = FormInputField(username))

    fun updateWithErrors(errors: List<InvalidDataError>): EditApiAccessFormData {
        var ret = this
        errors.forEach { ret = ret.setError(it) }
        return ret
    }

    private fun setError(error: InvalidDataError) =
        when (error) {
            is InvalidDataError.NameError -> copy(name = name.copy(error = error))
            is InvalidDataError.PasswordError -> copy(password = password.copy(error = error))
            is InvalidDataError.RemotePortError -> copy(remotePort = remotePort.copy(error = error))
            is InvalidDataError.ServerIpError -> copy(ip = ip.copy(error = error))
            is InvalidDataError.UserNameError -> copy(username = username.copy(error = error))
        }

    companion object {
        fun default(
            name: String = "",
            apiAccessMethodTypes: ApiAccessMethodTypes = ApiAccessMethodTypes.default(),
            ip: String = "",
            remotePort: String = "",
            enableAuthentication: Boolean = false,
            username: String = "",
            password: String = "",
            cipher: Cipher = Cipher.first()
        ) =
            EditApiAccessFormData(
                name = FormInputField(name),
                apiAccessMethodTypes = apiAccessMethodTypes,
                ip = FormInputField(ip),
                remotePort = FormInputField(remotePort),
                enableAuthentication = enableAuthentication,
                username = FormInputField(username),
                password = FormInputField(password),
                cipher = cipher
            )
    }
}

data class FormInputField<T>(val input: String, val error: T? = null)

enum class ApiAccessMethodTypes {
    SOCKS5_REMOTE,
    SHADOWSOCKS;

    companion object {
        fun default(): ApiAccessMethodTypes = SHADOWSOCKS
    }
}
