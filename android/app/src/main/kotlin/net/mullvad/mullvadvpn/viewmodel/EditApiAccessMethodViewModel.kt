package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.raise.either
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodTypes
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodInvalidDataErrors
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.GetApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.NewAccessMethod
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.SocksAuth
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodInput
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodUseCase
import net.mullvad.mullvadvpn.util.inAnyOf
import org.apache.commons.validator.routines.InetAddressValidator

class EditApiAccessMethodViewModel(
    private val apiAccessMethodId: ApiAccessMethodId?,
    private val apiAccessRepository: ApiAccessRepository,
    private val apiAccessMethodUseCase: TestApiAccessMethodUseCase,
    private val inetAddressValidator: InetAddressValidator
) : ViewModel() {
    private var initialData: EditApiAccessFormData? = null
    private val _sideEffects = Channel<EditApiAccessSideEffect>(Channel.BUFFERED)
    val sideEffect = _sideEffects.receiveAsFlow()
    private val testMethodState = MutableStateFlow<TestApiAccessMethodState?>(null)
    private val formData = MutableStateFlow<EditApiAccessFormData?>(null)
    val uiState =
        combine(formData, testMethodState) { formData, testMethodState ->
                formData?.let {
                    EditApiAccessMethodUiState.Content(
                        editMode = apiAccessMethodId != null,
                        formData = formData,
                        testMethodState = testMethodState
                    )
                } ?: EditApiAccessMethodUiState.Loading(editMode = apiAccessMethodId != null)
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                EditApiAccessMethodUiState.Loading(editMode = apiAccessMethodId != null)
            )

    init {
        // If we are editing the api access method, fetch initial data
        viewModelScope.launch {
            apiAccessMethodId?.let {
                apiAccessRepository
                    .getApiAccessMethodById(it)
                    .fold(::handleInitialDataError, ::setUpInitialData)
            }
                ?: run {
                    formData.value = EditApiAccessFormData.default()
                    initialData = EditApiAccessFormData.default()
                }
        }
    }

    fun setAccessMethodType(accessMethodType: ApiAccessMethodTypes) {
        formData.update { it?.copy(apiAccessMethodTypes = accessMethodType) }
    }

    fun updateName(name: String) {
        formData.update { it?.updateName(name) }
    }

    fun updateServerIp(serverIp: String) {
        formData.update { it?.updateServerIp(serverIp) }
    }

    fun updateRemotePort(port: String) {
        formData.update { it?.updateRemotePort(port) }
    }

    fun updatePassword(password: String) {
        formData.update { it?.updatePassword(password) }
    }

    fun updateCipher(cipher: Cipher) {
        formData.update { it?.updateCipher(cipher) }
    }

    fun updateAuthenticationEnabled(enabled: Boolean) {
        formData.update { it?.updateAuthenticationEnabled(enabled) }
    }

    fun updateUsername(username: String) {
        formData.update { it?.updateUsername(username) }
    }

    fun testMethod() {
        viewModelScope.launch {
            formData.value
                ?.validate()
                ?.fold(
                    { errors -> formData.update { it?.updateWithErrors(errors.errors) } },
                    { formData ->
                        apiAccessMethodUseCase
                            .testApiAccessMethod(
                                TestApiAccessMethodInput.TestNewMethod(formData.toCustomProxy())
                            )
                            .collect(testMethodState)
                    }
                )
        }
    }

    fun trySave() {
        viewModelScope.launch {
            formData.value
                ?.validate()
                ?.fold(
                    { errors -> formData.update { it?.updateWithErrors(errors.errors) } },
                    { formData ->
                        _sideEffects.send(
                            EditApiAccessSideEffect.OpenSaveDialog(formData.toNewAccessMethod())
                        )
                    }
                )
        }
    }

    fun onNavigateBack() {
        // Check if we have any unsaved changes
        viewModelScope.launch {
            if (initialData?.equals(formData.value) == true) {
                _sideEffects.send(EditApiAccessSideEffect.CLoseScreen)
            } else {
                _sideEffects.send(EditApiAccessSideEffect.ShowDiscardChangesDialog)
            }
        }
    }

    private fun handleInitialDataError(error: GetApiAccessMethodError) {
        viewModelScope.launch {
            _sideEffects.send(EditApiAccessSideEffect.UnableToGetApiAccessMethod(error))
        }
    }

    private fun setUpInitialData(accessMethod: ApiAccessMethod) {
        with(
            when (val customProxy = accessMethod.apiAccessMethodType) {
                ApiAccessMethodType.Bridges,
                ApiAccessMethodType.Direct ->
                    error("$customProxy api access type can not be edited")
                is ApiAccessMethodType.CustomProxy.Shadowsocks -> {
                    EditApiAccessFormData.default(
                        name = accessMethod.name.value,
                        ip = customProxy.ip,
                        remotePort = customProxy.port.toString(),
                        password = customProxy.password ?: "",
                        cipher = customProxy.cipher
                    )
                }
                is ApiAccessMethodType.CustomProxy.Socks5Remote ->
                    EditApiAccessFormData.default(
                        name = accessMethod.name.value,
                        ip = customProxy.ip,
                        remotePort = customProxy.port.toString(),
                        enableAuthentication = customProxy.auth != null,
                        username = customProxy.auth?.username ?: "",
                        password = customProxy.auth?.password ?: ""
                    )
            }
        ) {
            formData.value = this
            initialData = this
        }
    }

    private fun EditApiAccessFormData.validate() = either {
        val errors = mutableListOf<InvalidDataError>()
        if (name.input.isBlank()) {
            errors.add(InvalidDataError.NameError.Required)
        }
        when (this@validate.apiAccessMethodTypes) {
            ApiAccessMethodTypes.SHADOWSOCKS -> {
                errors.addAll(this@validate.validateShadowSocks())
            }
            ApiAccessMethodTypes.SOCKS5_REMOTE -> {
                errors.addAll(this@validate.validateSocks5Remote())
            }
        }
        if (errors.isNotEmpty()) {
            raise(ApiAccessMethodInvalidDataErrors(errors))
        }
        this@validate
    }

    private fun EditApiAccessFormData.validateShadowSocks(): List<InvalidDataError> {
        val errors = mutableListOf<InvalidDataError>()
        if (ip.input.isBlank()) {
            errors.add(InvalidDataError.ServerIpError.Required)
        } else if (!inetAddressValidator.isValid(ip.input)) {
            errors.add(InvalidDataError.ServerIpError.Invalid)
        }
        if (remotePort.input.isBlank()) {
            errors.add(InvalidDataError.RemotePortError.Required)
        } else if (!remotePort.input.validatePort()) {
            errors.add(InvalidDataError.RemotePortError.Invalid)
        }
        return errors
    }

    private fun EditApiAccessFormData.validateSocks5Remote(): List<InvalidDataError> {
        val errors = mutableListOf<InvalidDataError>()
        if (ip.input.isBlank()) {
            errors.add(InvalidDataError.ServerIpError.Required)
        } else if (!inetAddressValidator.isValid(ip.input)) {
            errors.add(InvalidDataError.ServerIpError.Invalid)
        }
        if (remotePort.input.isBlank()) {
            errors.add(InvalidDataError.RemotePortError.Required)
        } else if (!remotePort.input.validatePort()) {
            errors.add(InvalidDataError.RemotePortError.Invalid)
        }
        if (enableAuthentication) {
            if (username.input.isBlank()) {
                errors.add(InvalidDataError.UserNameError.Required)
            }
            if (password.input.isBlank()) {
                errors.add(InvalidDataError.PasswordError.Required)
            }
        }
        return errors
    }

    private fun EditApiAccessFormData.toCustomProxy(): ApiAccessMethodType.CustomProxy =
        when (this.apiAccessMethodTypes) {
            ApiAccessMethodTypes.SHADOWSOCKS ->
                ApiAccessMethodType.CustomProxy.Shadowsocks(
                    ip = ip.input,
                    port = Port.fromString(remotePort.input).getOrNull()!!,
                    password = password.input,
                    cipher = cipher
                )
            ApiAccessMethodTypes.SOCKS5_REMOTE ->
                ApiAccessMethodType.CustomProxy.Socks5Remote(
                    ip = ip.input,
                    port = Port.fromString(remotePort.input).getOrNull()!!,
                    auth =
                        if (enableAuthentication) {
                            SocksAuth(username.input, password.input)
                        } else {
                            null
                        },
                )
        }

    private fun EditApiAccessFormData.toNewAccessMethod(): NewAccessMethod =
        NewAccessMethod(
            name = ApiAccessMethodName.fromString(this.name.input),
            enabled = true,
            apiAccessMethodType = this.toCustomProxy()
        )

    private fun String.validatePort(): Boolean {
        return Port.fromString(this@validatePort).fold({ false }, { it.inAnyOf(allValidPorts) })
    }

    companion object {
        private val allValidPorts = listOf(PortRange(IntRange(0, 65535)))
    }
}

sealed interface EditApiAccessSideEffect {
    data class UnableToGetApiAccessMethod(val error: GetApiAccessMethodError) :
        EditApiAccessSideEffect

    data class OpenSaveDialog(val newAccessMethod: NewAccessMethod) : EditApiAccessSideEffect

    data object CLoseScreen : EditApiAccessSideEffect

    data object ShowDiscardChangesDialog : EditApiAccessSideEffect
}
