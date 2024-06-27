package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.Either.Companion.zipOrAccumulate
import arrow.core.EitherNel
import arrow.core.getOrElse
import arrow.core.nel
import arrow.core.raise.either
import arrow.core.raise.ensure
import com.ramcosta.composedestinations.generated.destinations.EditApiAccessMethodDestination
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodTypes
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.constant.MINIMUM_LOADING_TIME_MILLIS
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.SocksAuth
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.util.delayAtLeast
import org.apache.commons.validator.routines.InetAddressValidator

class EditApiAccessMethodViewModel(
    private val apiAccessRepository: ApiAccessRepository,
    private val inetAddressValidator: InetAddressValidator,
    savedStateHandle: SavedStateHandle
) : ViewModel() {
    private var testingJob: Job? = null
    private val apiAccessMethodId =
        EditApiAccessMethodDestination.argsFrom(savedStateHandle).accessMethodId

    private val _uiSideEffect = Channel<EditApiAccessSideEffect>(Channel.BUFFERED)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val isTestingApiAccessMethod = MutableStateFlow(false)
    private val formData = MutableStateFlow(initialData())
    val uiState =
        combine(flowOf(initialData()), formData, isTestingApiAccessMethod) {
                initialData,
                formData,
                isTestingApiAccessMethod ->
                EditApiAccessMethodUiState.Content(
                    editMode = apiAccessMethodId != null,
                    formData = formData,
                    hasChanges = initialData != formData,
                    isTestingApiAccessMethod = isTestingApiAccessMethod
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                EditApiAccessMethodUiState.Loading(editMode = apiAccessMethodId != null)
            )

    fun setAccessMethodType(accessMethodType: ApiAccessMethodTypes) {
        formData.update { it.copy(apiAccessMethodTypes = accessMethodType) }
    }

    fun onNameChanged(name: String) {
        formData.update { it.copy(name = name, nameError = null) }
    }

    fun onServerIpChanged(serverIp: String) {
        formData.update { it.copy(serverIp = serverIp, serverIpError = null) }
    }

    fun onPortChanged(port: String) {
        formData.update { it.copy(port = port, portError = null) }
    }

    fun onPasswordChanged(password: String) {
        formData.update { it.copy(password = password, passwordError = null) }
    }

    fun onCipherChanged(cipher: Cipher) {
        formData.update { it.copy(cipher = cipher) }
    }

    fun onAuthenticationEnabledChanged(enabled: Boolean) {
        formData.update { it.copy(enableAuthentication = enabled) }
    }

    fun onUsernameChanged(username: String) {
        formData.update { it.copy(username = username, usernameError = null) }
    }

    fun testMethod() {
        testingJob =
            viewModelScope.launch {
                formData.value
                    .parseConnectionFormData()
                    .fold(
                        { errors -> formData.update { it.updateWithErrors(errors) } },
                        { customProxy ->
                            isTestingApiAccessMethod.value = true
                            val result =
                                delayAtLeast(MINIMUM_LOADING_TIME_MILLIS) {
                                    apiAccessRepository.testCustomApiAccessMethod(customProxy)
                                }
                            _uiSideEffect.send(
                                EditApiAccessSideEffect.TestApiAccessMethodResult(result.isRight())
                            )
                            isTestingApiAccessMethod.value = false
                        }
                    )
            }
    }

    fun trySave() {
        viewModelScope.launch {
            formData.value
                .parseFormData()
                .fold(
                    { errors -> formData.update { it.updateWithErrors(errors) } },
                    { (name, customProxy) ->
                        _uiSideEffect.send(
                            EditApiAccessSideEffect.OpenSaveDialog(
                                id = apiAccessMethodId,
                                name = name,
                                customProxy = customProxy
                            )
                        )
                    }
                )
        }
    }

    fun cancelTestMethod() {
        if (testingJob?.isActive == true) {
            testingJob?.cancel("User cancelled test")
            isTestingApiAccessMethod.value = false
        }
    }

    private fun initialData(): EditApiAccessFormData =
        if (apiAccessMethodId == null) {
            EditApiAccessFormData.empty()
        } else {
            apiAccessRepository
                .getApiAccessMethodSettingById(apiAccessMethodId)
                .map { accessMethod ->
                    EditApiAccessFormData.fromCustomProxy(
                        accessMethod.name,
                        accessMethod.apiAccessMethod as? ApiAccessMethod.CustomProxy
                            ?: error(
                                "${accessMethod.apiAccessMethod} api access type can not be edited"
                            )
                    )
                }
                .getOrElse { error("Access method with id $apiAccessMethodId not found") }
        }

    private fun EditApiAccessFormData.parseFormData():
        EitherNel<InvalidDataError, Pair<ApiAccessMethodName, ApiAccessMethod.CustomProxy>> =
        zipOrAccumulate(parseName(name), parseConnectionFormData()) { name, customProxy ->
            name to customProxy
        }

    private fun EditApiAccessFormData.parseConnectionFormData() =
        when (apiAccessMethodTypes) {
            ApiAccessMethodTypes.SHADOWSOCKS -> {
                parseShadowSocksFormData(this)
            }
            ApiAccessMethodTypes.SOCKS5_REMOTE -> {
                parseSocks5RemoteFormData(this)
            }
        }

    private fun parseShadowSocksFormData(
        formData: EditApiAccessFormData
    ): EitherNel<InvalidDataError, ApiAccessMethod.CustomProxy.Shadowsocks> =
        parseIpAndPort(formData.serverIp, formData.port).map { (ip, port) ->
            ApiAccessMethod.CustomProxy.Shadowsocks(
                ip = ip,
                port = port,
                password = formData.password.ifBlank { null },
                cipher = formData.cipher
            )
        }

    private fun parseIpAddress(input: String): Either<InvalidDataError.ServerIpError, String> =
        either {
            ensure(input.isNotBlank()) { InvalidDataError.ServerIpError.Required }
            ensure(inetAddressValidator.isValid(input)) { InvalidDataError.ServerIpError.Invalid }
            input
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
    ): EitherNel<InvalidDataError, ApiAccessMethod.CustomProxy.Socks5Remote> =
        zipOrAccumulate(
            parseIpAndPort(formData.serverIp, formData.port),
            parseAuth(
                authEnabled = formData.enableAuthentication,
                inputUsername = formData.username,
                inputPassword = formData.password
            )
        ) { (ip, port), auth ->
            ApiAccessMethod.CustomProxy.Socks5Remote(ip = ip, port = port, auth = auth)
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
    ): EitherNel<InvalidDataError, SocksAuth?> =
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
        either {
            ensure(input.isNotBlank()) { InvalidDataError.UserNameError.Required }
            input
        }

    private fun parsePassword(input: String): Either<InvalidDataError.PasswordError, String> =
        either {
            ensure(input.isNotBlank()) { InvalidDataError.PasswordError.Required }
            input
        }

    private fun parseName(
        input: String
    ): EitherNel<InvalidDataError.NameError, ApiAccessMethodName> = either {
        ensure(input.isNotBlank()) { InvalidDataError.NameError.Required.nel() }
        ApiAccessMethodName.fromString(input)
    }
}

sealed interface EditApiAccessSideEffect {
    data class OpenSaveDialog(
        val id: ApiAccessMethodId?,
        val name: ApiAccessMethodName,
        val customProxy: ApiAccessMethod.CustomProxy
    ) : EditApiAccessSideEffect

    data class TestApiAccessMethodResult(val successful: Boolean) : EditApiAccessSideEffect
}
