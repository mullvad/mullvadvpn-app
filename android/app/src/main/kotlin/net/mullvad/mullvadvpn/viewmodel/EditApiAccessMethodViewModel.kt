package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.raise.either
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
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
import net.mullvad.mullvadvpn.compose.state.TestMethodState
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
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.util.inAnyOf
import org.apache.commons.validator.routines.InetAddressValidator

class EditApiAccessMethodViewModel(
    private val apiAccessMethodId: ApiAccessMethodId?,
    private val apiAccessRepository: ApiAccessRepository,
    private val inetAddressValidator: InetAddressValidator
) : ViewModel() {
    private val _sideEffects = Channel<EditApiAccessSideEffect>()
    val sideEffect = _sideEffects.receiveAsFlow()
    private val testMethodState = MutableStateFlow<TestMethodState?>(null)
    private val formatErrors = MutableStateFlow<ApiAccessMethodInvalidDataErrors?>(null)
    private val formData = MutableStateFlow<EditApiAccessFormData?>(null)
    val uiState =
        combine(formData, formatErrors, testMethodState) { formData, formatErrors, testMethodState
                ->
                formData?.let {
                    EditApiAccessMethodUiState.Content(
                        editMode = apiAccessMethodId != null,
                        formData = formData,
                        formErrors = formatErrors,
                        testMethodState = testMethodState
                    )
                } ?: EditApiAccessMethodUiState.Loading(editMode = apiAccessMethodId != null)
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                EditApiAccessMethodUiState.Loading(editMode = apiAccessMethodId != null)
            )

    private var testAndSaveJob: Job? = null

    init {
        // If we are editing the api access method, fetch initial data
        viewModelScope.launch {
            apiAccessMethodId?.let {
                apiAccessRepository
                    .getApiAccessMethodById(it)
                    .fold(::handleInitialDataError, ::setUpInitialData)
            } ?: run { formData.value = EditApiAccessFormData.empty() }
        }
    }

    fun setAccessMethodType(accessMethodType: ApiAccessMethodTypes) {
        val name = formData.value?.name
        // Currently we clear all old data except name when changing access method
        formData.value =
            EditApiAccessFormData.emptyFromTypeAndName(type = accessMethodType, name = name)
    }

    fun updateName(name: ApiAccessMethodName) {
        formatErrors.value = null
        formData.update { it?.updateName(name) }
    }

    fun updateServerIp(serverIp: String) {
        formatErrors.value = null
        formData.update { it?.updateServerIp(serverIp) }
    }

    fun updateRemotePort(port: Port?) {
        formatErrors.value = null
        formData.update { it?.updateRemotePort(port) }
    }

    fun updateLocalPort(port: Port?) {
        formatErrors.value = null
        formData.update { it?.updateLocalPort(port) }
    }

    fun updatePassword(password: String) {
        formatErrors.value = null
        formData.update { it?.updatePassword(password) }
    }

    fun updateCipher(cipher: Cipher) {
        formatErrors.value = null
        formData.update { it?.updateCipher(cipher) }
    }

    fun updateAuthenticationEnabled(enabled: Boolean) {
        formatErrors.value = null
        formData.update { it?.updateAuthenticationEnabled(enabled) }
    }

    fun updateUsername(username: String) {
        formatErrors.value = null
        formData.update { it?.updateUsername(username) }
    }

    fun updateTransportProtocol(transportProtocol: TransportProtocol) {
        formatErrors.value = null
        formData.update { it?.updateTransportProtocol(transportProtocol) }
    }

    fun testMethod() {
        viewModelScope.launch {
            formData.value
                ?.validate()
                ?.fold(
                    { formatErrors.value = it },
                    { formData ->
                        testMethodState.emit(TestMethodState.Testing)
                        apiAccessRepository
                            .testCustomApiAccessMethod(formData.toCustomProxy())
                            .fold(
                                { testMethodState.emit(TestMethodState.Failed) },
                                { testMethodState.emit(TestMethodState.Successful) }
                            )
                        delay(TEST_METHOD_RESULT_TIME_MS)
                        testMethodState.emit(null)
                    }
                )
        }
    }

    fun trySave() {
        viewModelScope.launch {
            formData.value
                ?.validate()
                ?.fold(
                    { formatErrors.value = it },
                    { formData ->
                        _sideEffects.send(
                            EditApiAccessSideEffect.OpenSaveDialog(formData.toNewAccessMethod())
                        )
                    }
                )
        }
    }

    private fun handleInitialDataError(error: GetApiAccessMethodError) {
        viewModelScope.launch {
            _sideEffects.send(EditApiAccessSideEffect.UnableToGetApiAccessMethod)
        }
    }

    private fun setUpInitialData(accessMethod: ApiAccessMethod) {
        formData.value =
            when (val customProxy = accessMethod.apiAccessMethodType) {
                ApiAccessMethodType.Bridges,
                ApiAccessMethodType.Direct ->
                    error("$customProxy api access type can not be edited")
                is ApiAccessMethodType.CustomProxy.Shadowsocks -> {
                    EditApiAccessFormData.Shadowsocks(
                        name = accessMethod.name,
                        ip = customProxy.ip,
                        port = customProxy.port,
                        password = customProxy.password,
                        cipher = customProxy.cipher
                    )
                }
                is ApiAccessMethodType.CustomProxy.Socks5Local ->
                    EditApiAccessFormData.Socks5Local(
                        name = accessMethod.name,
                        remotePort = customProxy.remotePort,
                        remoteIp = customProxy.remoteIp,
                        localPort = customProxy.localPort,
                        remoteTransportProtocol = customProxy.remoteTransportProtocol
                    )
                is ApiAccessMethodType.CustomProxy.Socks5Remote ->
                    EditApiAccessFormData.Socks5Remote(
                        name = accessMethod.name,
                        ip = customProxy.ip,
                        port = customProxy.port,
                        enableAuthentication = customProxy.auth != null,
                        username = customProxy.auth?.username,
                        password = customProxy.auth?.password
                    )
            }
    }

    private fun EditApiAccessFormData.validate() = either {
        val errors = mutableListOf<InvalidDataError>()
        if (name == null || name?.value.isNullOrBlank()) {
            errors.add(InvalidDataError.NameError.Required)
        }
        when (val format = this@validate) {
            is EditApiAccessFormData.Shadowsocks -> {
                errors.addAll(format.validate())
            }
            is EditApiAccessFormData.Socks5Local -> {
                errors.addAll(format.validate())
            }
            is EditApiAccessFormData.Socks5Remote -> {
                errors.addAll(format.validate())
            }
        }
        if (errors.isNotEmpty()) {
            raise(ApiAccessMethodInvalidDataErrors(errors))
        }
        this@validate
    }

    private fun EditApiAccessFormData.Shadowsocks.validate(): List<InvalidDataError> {
        val errors = mutableListOf<InvalidDataError>()
        if (ip.isNullOrBlank()) {
            errors.add(InvalidDataError.ServerIpError.Required)
        } else if (!inetAddressValidator.isValid(ip)) {
            errors.add(InvalidDataError.ServerIpError.Invalid)
        }
        if (port == null) {
            errors.add(InvalidDataError.RemotePortError.Required)
        } else if (!port.inAnyOf(allValidPorts)) {
            errors.add(InvalidDataError.RemotePortError.Invalid)
        }
        return errors
    }

    private fun EditApiAccessFormData.Socks5Local.validate(): List<InvalidDataError> {
        val errors = mutableListOf<InvalidDataError>()
        if (localPort == null) {
            errors.add(InvalidDataError.LocalPortError.Required)
        } else if (!localPort.inAnyOf(allValidPorts)) {
            errors.add(InvalidDataError.LocalPortError.Invalid)
        }
        if (remotePort == null) {
            errors.add(InvalidDataError.RemotePortError.Required)
        } else if (!remotePort.inAnyOf(allValidPorts)) {
            errors.add(InvalidDataError.RemotePortError.Invalid)
        }
        if (remoteIp.isNullOrBlank()) {
            errors.add(InvalidDataError.ServerIpError.Required)
        } else if (!inetAddressValidator.isValid(remoteIp)) {
            errors.add(InvalidDataError.ServerIpError.Invalid)
        }
        return errors
    }

    private fun EditApiAccessFormData.Socks5Remote.validate(): List<InvalidDataError> {
        val errors = mutableListOf<InvalidDataError>()
        if (ip.isNullOrBlank()) {
            errors.add(InvalidDataError.ServerIpError.Required)
        } else if (!inetAddressValidator.isValid(ip)) {
            errors.add(InvalidDataError.ServerIpError.Invalid)
        }
        if (port == null) {
            errors.add(InvalidDataError.RemotePortError.Required)
        } else if (!port.inAnyOf(allValidPorts)) {
            errors.add(InvalidDataError.RemotePortError.Invalid)
        }
        if (enableAuthentication) {
            if (username.isNullOrBlank()) {
                errors.add(InvalidDataError.UserNameError.Required)
            }
            if (password.isNullOrBlank()) {
                errors.add(InvalidDataError.PasswordError.Required)
            }
        }
        return errors
    }

    private fun EditApiAccessFormData.toCustomProxy(): ApiAccessMethodType.CustomProxy =
        when (this) {
            is EditApiAccessFormData.Shadowsocks ->
                ApiAccessMethodType.CustomProxy.Shadowsocks(
                    ip = ip!!,
                    port = port!!,
                    password = password,
                    cipher = cipher
                )
            is EditApiAccessFormData.Socks5Local -> {
                ApiAccessMethodType.CustomProxy.Socks5Local(
                    remoteIp = remoteIp!!,
                    remotePort = remotePort!!,
                    remoteTransportProtocol = remoteTransportProtocol,
                    localPort = localPort!!
                )
            }
            is EditApiAccessFormData.Socks5Remote ->
                ApiAccessMethodType.CustomProxy.Socks5Remote(
                    ip = ip!!,
                    port = port!!,
                    auth =
                        if (enableAuthentication) {
                            SocksAuth(username!!, password!!)
                        } else {
                            null
                        },
                )
        }

    private fun EditApiAccessFormData.toNewAccessMethod(): NewAccessMethod =
        NewAccessMethod(
            name = this.name!!,
            enabled = true,
            apiAccessMethodType = this.toCustomProxy()
        )

    companion object {
        private val allValidPorts = listOf(PortRange(IntRange(0, 65535)))
        private const val TEST_METHOD_RESULT_TIME_MS = 1000L * 5
    }
}

sealed interface EditApiAccessSideEffect {
    data object UnableToGetApiAccessMethod : EditApiAccessSideEffect

    data class OpenSaveDialog(val newAccessMethod: NewAccessMethod) : EditApiAccessSideEffect
}
