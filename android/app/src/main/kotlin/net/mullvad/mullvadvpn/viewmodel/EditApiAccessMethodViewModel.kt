package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.Either.Companion.zipOrAccumulate
import arrow.core.EitherNel
import arrow.core.NonEmptyList
import arrow.core.left
import arrow.core.leftNel
import arrow.core.right
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
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodType
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.GetApiAccessMethodError
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.SocksAuth
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodInput
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodUseCase
import org.apache.commons.validator.routines.InetAddressValidator

class EditApiAccessMethodViewModel(
    private val apiAccessMethodId: ApiAccessMethodId?,
    private val apiAccessRepository: ApiAccessRepository,
    private val apiAccessMethodUseCase: TestApiAccessMethodUseCase,
    private val inetAddressValidator: InetAddressValidator
) : ViewModel() {
    private var enabled: Boolean = true
    private var initialData: EditApiAccessFormData? = null
    private val _uiSideEffect = Channel<EditApiAccessSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
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
            apiAccessMethodId?.let { apiAccessMethodId ->
                apiAccessRepository
                    .getApiAccessMethodById(apiAccessMethodId)
                    .onRight { enabled = it.enabled }
                    .fold(::handleInitialDataError, ::setUpInitialData)
            }
                ?: run {
                    formData.value = EditApiAccessFormData.empty()
                    initialData = EditApiAccessFormData.empty()
                    enabled = true
                }
        }
    }

    fun setAccessMethodType(accessMethodType: ApiAccessMethodTypes) {
        formData.update { it?.copy(apiAccessMethodTypes = accessMethodType) }
    }

    fun onNameChanged(name: String) {
        formData.update { it?.copy(name = name, nameError = null) }
    }

    fun onServerIpChanged(serverIp: String) {
        formData.update { it?.copy(serverIp = serverIp, serverIpError = null) }
    }

    fun onPortChanged(port: String) {
        formData.update { it?.copy(port = port, portError = null) }
    }

    fun onPasswordChanged(password: String) {
        formData.update { it?.copy(password = password, passwordError = null) }
    }

    fun onCipherChanged(cipher: Cipher) {
        formData.update { it?.copy(cipher = cipher) }
    }

    fun onAuthenticationEnabledChanged(enabled: Boolean) {
        formData.update { it?.copy(enableAuthentication = enabled) }
    }

    fun onUsernameChanged(username: String) {
        formData.update { it?.copy(username = username, usernameError = null) }
    }

    fun testMethod() {
        viewModelScope.launch {
            formData.value
                ?.parseFormData(skipNameValidation = true)
                ?.fold(
                    { errors -> formData.update { it?.updateWithErrors(errors) } },
                    { (_, customProxy) ->
                        apiAccessMethodUseCase
                            .testApiAccessMethod(
                                TestApiAccessMethodInput.TestNewMethod(customProxy)
                            )
                            .collect(testMethodState)
                    }
                )
        }
    }

    fun trySave() {
        viewModelScope.launch {
            formData.value
                ?.parseFormData(skipNameValidation = false)
                ?.fold(
                    { errors -> formData.update { it?.updateWithErrors(errors) } },
                    { (name, customProxy) ->
                        _uiSideEffect.send(
                            EditApiAccessSideEffect.OpenSaveDialog(
                                id = apiAccessMethodId,
                                name = name,
                                enabled = enabled,
                                customProxy = customProxy
                            )
                        )
                    }
                )
        }
    }

    fun onNavigateBack() {
        // Check if we have any unsaved changes
        viewModelScope.launch {
            if (initialData == formData.value) {
                _uiSideEffect.send(EditApiAccessSideEffect.CloseScreen)
            } else {
                _uiSideEffect.send(EditApiAccessSideEffect.ShowDiscardChangesDialog)
            }
        }
    }

    private fun handleInitialDataError(error: GetApiAccessMethodError) {
        viewModelScope.launch {
            _uiSideEffect.send(EditApiAccessSideEffect.UnableToGetApiAccessMethod(error))
        }
    }

    private fun setUpInitialData(accessMethod: ApiAccessMethod) {
        val initialFormData =
            EditApiAccessFormData.fromCustomProxy(
                accessMethod.name,
                accessMethod.apiAccessMethodType as? ApiAccessMethodType.CustomProxy
                    ?: error(
                        "${accessMethod.apiAccessMethodType} api access type can not be edited"
                    )
            )
        with(initialFormData) {
            formData.value = this
            initialData = this
        }
    }

    private fun EditApiAccessFormData.parseFormData(
        skipNameValidation: Boolean
    ): Either<
        NonEmptyList<InvalidDataError>,
        Pair<ApiAccessMethodName, ApiAccessMethodType.CustomProxy>
    > =
        zipOrAccumulate(
            parseName(name, skipNameValidation),
            when (apiAccessMethodTypes) {
                ApiAccessMethodTypes.SHADOWSOCKS -> {
                    parseShadowSocksFormData(this)
                }
                ApiAccessMethodTypes.SOCKS5_REMOTE -> {
                    parseSocks5RemoteFormData(this)
                }
            }
        ) { name, customProxy ->
            name to customProxy
        }

    private fun parseShadowSocksFormData(
        formData: EditApiAccessFormData
    ): EitherNel<InvalidDataError, ApiAccessMethodType.CustomProxy.Shadowsocks> =
        parseIpAndPort(formData.serverIp, formData.port).map { (ip, port) ->
            ApiAccessMethodType.CustomProxy.Shadowsocks(
                ip = ip,
                port = port,
                password = formData.password.ifBlank { null },
                cipher = formData.cipher
            )
        }

    private fun parseIpAddress(input: String): Either<InvalidDataError.ServerIpError, String> =
        when {
            input.isBlank() -> InvalidDataError.ServerIpError.Required.left()
            !inetAddressValidator.isValid(input) -> InvalidDataError.ServerIpError.Invalid.left()
            else -> input.right()
        }

    private fun parsePort(input: String): Either<InvalidDataError.PortError, Port> =
        Port.fromString(input).mapLeft {
            when (it) {
                is ParsePortError.NotANumber ->
                    if (it.input.isBlank()) {
                        InvalidDataError.PortError.Required
                    } else {
                        InvalidDataError.PortError.Invalid(it)
                    }
                is ParsePortError.OutOfRange -> InvalidDataError.PortError.Invalid(it)
            }
        }

    private fun parseSocks5RemoteFormData(
        formData: EditApiAccessFormData
    ): Either<NonEmptyList<InvalidDataError>, ApiAccessMethodType.CustomProxy.Socks5Remote> =
        zipOrAccumulate(
            parseIpAndPort(formData.serverIp, formData.port),
            parseAuth(
                authEnabled = formData.enableAuthentication,
                inputUsername = formData.username,
                inputPassword = formData.password
            )
        ) { (ip, port), auth ->
            ApiAccessMethodType.CustomProxy.Socks5Remote(ip = ip, port = port, auth = auth)
        }

    private fun parseIpAndPort(ipInput: String, portInput: String) =
        zipOrAccumulate(
            parseIpAddress(ipInput),
            parsePort(portInput),
        ) { ip, port ->
            ip to port
        }

    private fun parseAuth(
        authEnabled: Boolean,
        inputUsername: String,
        inputPassword: String
    ): Either<NonEmptyList<InvalidDataError>, SocksAuth?> =
        if (!authEnabled) {
            Either.Right(null)
        } else {
            zipOrAccumulate(parseUsername(inputUsername), parsePassword(inputPassword)) {
                userName,
                password ->
                SocksAuth(userName, password)
            }
        }

    private fun parseUsername(input: String): Either<InvalidDataError.UserNameError, String> =
        if (input.isBlank()) {
            InvalidDataError.UserNameError.Required.left()
        } else {
            input.right()
        }

    private fun parsePassword(input: String): Either<InvalidDataError.PasswordError, String> =
        if (input.isBlank()) {
            InvalidDataError.PasswordError.Required.left()
        } else {
            input.right()
        }

    private fun parseName(
        input: String,
        noError: Boolean
    ): EitherNel<InvalidDataError.NameError, ApiAccessMethodName> =
        if (input.isBlank() && !noError) {
            InvalidDataError.NameError.Required.leftNel()
        } else {
            ApiAccessMethodName.fromString(input).right()
        }
}

sealed interface EditApiAccessSideEffect {
    data class UnableToGetApiAccessMethod(val error: GetApiAccessMethodError) :
        EditApiAccessSideEffect

    data class OpenSaveDialog(
        val id: ApiAccessMethodId?,
        val name: ApiAccessMethodName,
        val enabled: Boolean,
        val customProxy: ApiAccessMethodType.CustomProxy
    ) : EditApiAccessSideEffect

    data object CloseScreen : EditApiAccessSideEffect

    data object ShowDiscardChangesDialog : EditApiAccessSideEffect
}
