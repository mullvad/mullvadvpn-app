package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import arrow.core.nonEmptyListOf
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.InvalidDataError

class EditApiAccessMethodUiStatePreviewParameterProvider :
    PreviewParameterProvider<EditApiAccessMethodUiState> {
    override val values =
        sequenceOf(
            EditApiAccessMethodUiState.Loading(editMode = true),
            // Empty default state
            EditApiAccessMethodUiState.Content(
                editMode = false,
                formData = EditApiAccessFormData.empty(),
                hasChanges = false,
                isTestingApiAccessMethod = false,
            ),
            // Shadowsocks, no errors
            EditApiAccessMethodUiState.Content(
                editMode = true,
                hasChanges = false,
                formData =
                    shadowsocks.let {
                        val data = (it.apiAccessMethod as ApiAccessMethod.CustomProxy.Shadowsocks)
                        EditApiAccessFormData(
                            name = it.name.value,
                            serverIp = data.ip,
                            port = data.port.toString(),
                            password = data.password ?: "",
                            cipher = data.cipher,
                            username = "",
                        )
                    },
                isTestingApiAccessMethod = false,
            ),
            // Socks5 Remote, no errors, testing method
            EditApiAccessMethodUiState.Content(
                editMode = true,
                hasChanges = false,
                formData =
                    socks5Remote.let {
                        val data = (it.apiAccessMethod as ApiAccessMethod.CustomProxy.Socks5Remote)
                        EditApiAccessFormData(
                            name = it.name.value,
                            serverIp = data.ip,
                            port = data.port.toString(),
                            enableAuthentication = data.auth != null,
                            username = data.auth?.username ?: "",
                            password = data.auth?.password ?: "",
                        )
                    },
                isTestingApiAccessMethod = true,
            ),
            // Socks 5 remote, required errors
            EditApiAccessMethodUiState.Content(
                editMode = true,
                hasChanges = false,
                formData =
                    EditApiAccessFormData.empty()
                        .copy(enableAuthentication = true)
                        .updateWithErrors(
                            nonEmptyListOf(
                                InvalidDataError.NameError.Required,
                                InvalidDataError.PortError.Required,
                                InvalidDataError.ServerIpError.Required,
                                InvalidDataError.UserNameError.Required,
                                InvalidDataError.PasswordError.Required,
                            )
                        ),
                isTestingApiAccessMethod = false,
            ),
        )
}
