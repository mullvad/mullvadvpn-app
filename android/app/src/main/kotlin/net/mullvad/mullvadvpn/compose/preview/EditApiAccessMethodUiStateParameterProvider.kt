package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState

class EditApiAccessMethodUiStateParameterProvider :
    PreviewParameterProvider<EditApiAccessMethodUiState> {
    override val values =
        sequenceOf(
            EditApiAccessMethodUiState.Loading(editMode = true),
            // Empty default state
            EditApiAccessMethodUiState.Content(
                editMode = false,
                formData = EditApiAccessFormData.default(),
                testMethodState = null
            ),
            // Shadowsocks, no errors
            EditApiAccessMethodUiState.Content(
                editMode = true,
                formData =
                    shadowsocks.let {
                        val data =
                            (it.apiAccessMethodType as ApiAccessMethodType.CustomProxy.Shadowsocks)
                        EditApiAccessFormData.default(
                            name = it.name.value,
                            ip = data.ip,
                            remotePort = data.port.toString(),
                            password = data.password ?: "",
                            cipher = data.cipher
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
                        EditApiAccessFormData.default(
                            name = it.name.value,
                            ip = data.ip,
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
                    EditApiAccessFormData.default(enableAuthentication = true)
                        .updateWithErrors(
                            listOf(
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
