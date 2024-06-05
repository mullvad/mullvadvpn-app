package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import arrow.core.nonEmptyListOf
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState

class EditApiAccessMethodUiStateParameterProvider :
    PreviewParameterProvider<EditApiAccessMethodUiState> {
    override val values =
        sequenceOf(
            EditApiAccessMethodUiState.Loading(editMode = true),
            // Empty default state
            EditApiAccessMethodUiState.Content(
                editMode = false,
                formData = EditApiAccessFormData.empty(),
                testMethodState = null
            ),
            // Shadowsocks, no errors
            EditApiAccessMethodUiState.Content(
                editMode = true,
                formData =
                    shadowsocks.let {
                        val data =
                            (it.apiAccessMethodType as ApiAccessMethodType.CustomProxy.Shadowsocks)
                        EditApiAccessFormData(
                            name = it.name.value,
                            serverIp = data.ip,
                            remotePort = data.port.toString(),
                            password = data.password ?: "",
                            cipher = data.cipher,
                            username = ""
                        )
                    },
                testMethodState = null
            ),
            // Socks5 Remote, no errors, testing method
            EditApiAccessMethodUiState.Content(
                editMode = true,
                formData =
                    socks5Remote.let {
                        val data =
                            (it.apiAccessMethodType as ApiAccessMethodType.CustomProxy.Socks5Remote)
                        EditApiAccessFormData(
                            name = it.name.value,
                            serverIp = data.ip,
                            remotePort = data.port.toString(),
                            enableAuthentication = data.auth != null,
                            username = data.auth?.username ?: "",
                            password = data.auth?.password ?: ""
                        )
                    },
                testMethodState = TestApiAccessMethodState.Testing
            ),
            // Socks 5 remote, required errors
            EditApiAccessMethodUiState.Content(
                editMode = true,
                formData =
                    EditApiAccessFormData.empty()
                        .copy(enableAuthentication = true)
                        .updateWithErrors(
                            nonEmptyListOf(
                                InvalidDataError.NameError.Required,
                                InvalidDataError.RemotePortError.Required,
                                InvalidDataError.ServerIpError.Required,
                                InvalidDataError.UserNameError.Required,
                                InvalidDataError.PasswordError.Required
                            )
                        ),
                testMethodState = null
            )
        )
}
