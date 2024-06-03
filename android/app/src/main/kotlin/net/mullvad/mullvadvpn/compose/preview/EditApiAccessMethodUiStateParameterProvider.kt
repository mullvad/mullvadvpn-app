package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodInvalidDataErrors
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState

class EditApiAccessMethodUiStateParameterProvider :
    PreviewParameterProvider<EditApiAccessMethodUiState> {
    override val values =
        sequenceOf(
            EditApiAccessMethodUiState.Loading(editMode = true),
            // Empty default state
            EditApiAccessMethodUiState.Content(
                editMode = false,
                formData = EditApiAccessFormData.empty(),
                formErrors = null,
                testMethodState = null
            ),
            // Shadowsocks, no errors
            EditApiAccessMethodUiState.Content(
                editMode = true,
                formData =
                    shadowsocks.let {
                        val data =
                            (it.apiAccessMethodType as ApiAccessMethodType.CustomProxy.Shadowsocks)
                        EditApiAccessFormData.Shadowsocks(
                            name = it.name,
                            ip = data.ip,
                            port = data.port,
                            password = data.password,
                            cipher = data.cipher
                        )
                    },
                formErrors = null,
                testMethodState = null
            ),
            // Socks5 Remote, no errors, testing method
            EditApiAccessMethodUiState.Content(
                editMode = true,
                formData =
                    socks5Remote.let {
                        val data =
                            (it.apiAccessMethodType as ApiAccessMethodType.CustomProxy.Socks5Remote)
                        EditApiAccessFormData.Socks5Remote(
                            name = it.name,
                            ip = data.ip,
                            port = data.port,
                            enableAuthentication = data.auth != null,
                            username = data.auth?.username,
                            password = data.auth?.password
                        )
                    },
                formErrors = null,
                testMethodState = TestApiAccessMethodState.Testing
            ),
            // Socks5 Local, input required errors
            EditApiAccessMethodUiState.Content(
                editMode = true,
                formData =
                    EditApiAccessFormData.Socks5Local(
                        remoteTransportProtocol = TransportProtocol.Tcp
                    ),
                formErrors =
                    ApiAccessMethodInvalidDataErrors(
                        errors =
                            listOf(
                                InvalidDataError.NameError.Required,
                                InvalidDataError.LocalPortError.Required,
                                InvalidDataError.RemotePortError.Required,
                                InvalidDataError.ServerIpError.Required
                            )
                    ),
                testMethodState = null
            )
        )
}
