package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import java.net.InetAddress
import java.net.InetSocketAddress
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.common.util.onFirst
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.PersonalVpnConfig
import net.mullvad.mullvadvpn.lib.model.KeyParseError
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.model.WireguardKey
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

class PersonalVpnViewModel(
    settingsRepository: SettingsRepository,
    val managementService: ManagementService,
) : ViewModel() {
    private val _uiSideEffect = Channel<PersonalVpnSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    private val _formData = MutableStateFlow<PersonalVpnFormData?>(null)
    private val formErrors = MutableStateFlow<List<FormDataError>>(emptyList())

    val uiState: StateFlow<Lc<Boolean, PersonalVpnUiState>> =
        combine(
                settingsRepository.settingsUpdates
                    .filterNotNull()
                    .onFirst { _formData.value = PersonalVpnFormData.from(it.personalVpnConfig) }
                    .map { it.personalVpnEnabled },
                managementService.tunnelStats.onStart { emit(TunnelStats()) },
                _formData.filterNotNull(),
                formErrors,
                _formData.map { it != PersonalVpnFormData() },
            ) {
                personalVpnEnabled: Boolean,
                personalVpnStats: TunnelStats,
                formData: PersonalVpnFormData,
                formErrors,
                canClear ->
                PersonalVpnUiState(
                        enabled = personalVpnEnabled,
                        canClear,
                        tunnelStats = personalVpnStats,
                        formData,
                        formErrors.filterIsInstance<FormDataError.PrivateKey>().firstOrNull(),
                        formErrors.filterIsInstance<FormDataError.TunnelIp>().firstOrNull(),
                        formErrors.filterIsInstance<FormDataError.PublicKey>().firstOrNull(),
                        formErrors
                            .filterIsInstance<FormDataError.AllowedIp>()
                            .associateBy { it.index },
                        formErrors.filterIsInstance<FormDataError.Endpoint>().firstOrNull(),
                    )
                    .toLc<Boolean, PersonalVpnUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(false),
            )

    fun onToggle(on: Boolean) {
        viewModelScope.launch { managementService.togglePersonalVpn(on) }
    }

    fun onClearError(error: FormDataError) {
        formErrors.update { it - error }
    }

    fun onClearAllowedIpErrors() {
        formErrors.update { errors -> errors.filterNot { it is FormDataError.AllowedIp } }
    }

    fun clearConfig() {
        viewModelScope.launch {
            managementService.clearPersonalVpn()
            managementService.togglePersonalVpn(false)
            _formData.value = PersonalVpnFormData.from(null)
        }
    }

    fun save(formData: PersonalVpnFormData) = viewModelScope.launch {
        parseFormData(formData)
            .fold(
                {
                    formErrors.value = it
                    _uiSideEffect.send(PersonalVpnSideEffect.FormErrors)
                },
                { vpnConfig ->
                    formErrors.value = emptyList()
                    viewModelScope.launch {
                        managementService
                            .setPersonalVpnConfig(vpnConfig)
                            .fold(
                                {
                                    _uiSideEffect.send(
                                        PersonalVpnSideEffect.FailedToSave(it.toString())
                                    )
                                },
                                {
                                    _formData.value = PersonalVpnFormData.from(vpnConfig)
                                    _uiSideEffect.send(PersonalVpnSideEffect.ConfigurationSaved)
                                },
                            )
                    }
                },
            )
    }

    private fun parseFormData(
        formData: PersonalVpnFormData
    ): Either<List<FormDataError>, PersonalVpnConfig> {
        val errors = mutableListOf<FormDataError>()

        val privateKey = parsePrivateKey(formData.privateKey).onLeft { errors.add(it) }.getOrNull()
        val address = parseAddress(formData.tunnelIp).onLeft { errors.add(it) }.getOrNull()
        val publicKey = parsePublicKey(formData.publicKey).onLeft { errors.add(it) }.getOrNull()
        val allowedIps =
            parseAllowedIps(formData.allowedIPs).onLeft { errors.addAll(it) }.getOrNull()
        val endpoint = parseEndpoint(formData.endpoint).onLeft { errors.add(it) }.getOrNull()

        if (errors.isNotEmpty()) return Either.Left(errors)

        return Either.Right(
            PersonalVpnConfig(
                tunnelConfig = TunnelConfig(privateKey = privateKey!!, tunnelIp = address!!),
                peerConfig =
                    PeerConfig(
                        publicKey = publicKey!!,
                        allowedIps = allowedIps!!,
                        endpoint = endpoint!!,
                    ),
            )
        )
    }

    private fun parsePrivateKey(
        privateKey: String
    ): Either<FormDataError.PrivateKey, WireguardKey> {
        return WireguardKey.from(privateKey).mapLeft { FormDataError.PrivateKey(it) }
    }

    private fun parseAddress(address: String): Either<FormDataError.TunnelIp, InetAddress> =
        Either.catch { InetAddress.getByName(address) }.mapLeft { FormDataError.TunnelIp }

    private fun parsePublicKey(publicKey: String): Either<FormDataError.PublicKey, WireguardKey> {
        return WireguardKey.from(publicKey).mapLeft { FormDataError.PublicKey(it) }
    }

    private fun parseAllowedIps(
        allowedIPs: List<String>
    ): Either<List<FormDataError.AllowedIp>, List<String>> {
        val errors = mutableListOf<FormDataError.AllowedIp>()
        allowedIPs.forEachIndexed { index, ip ->
            if (ip.isBlank()) {
                errors.add(FormDataError.AllowedIp(index))
            }
        }
        return if (errors.isEmpty()) {
            Either.Right(allowedIPs)
        } else {
            Either.Left(errors)
        }
    }

    private fun parseEndpoint(endpoint: String): Either<FormDataError.Endpoint, InetSocketAddress> =
        either {
            ensure(endpoint.isNotBlank()) { FormDataError.Endpoint.Empty }
            ensure(endpoint.contains(':')) { FormDataError.Endpoint.InvalidAddress }
            val (rawHost, rawPort) = endpoint.trim().split(':')
            val host =
                Either.catch { InetAddress.getByName(rawHost) }
                    .mapLeft { FormDataError.Endpoint.InvalidAddress }
                    .bind()

            val port =
                Port.fromString(rawPort).mapLeft { FormDataError.Endpoint.InvalidPort(it) }.bind()

            InetSocketAddress(host, port.value)
        }
}

sealed interface PersonalVpnSideEffect {

    data object FormErrors : PersonalVpnSideEffect

    data object ConfigurationSaved : PersonalVpnSideEffect

    data class FailedToSave(val reason: String) : PersonalVpnSideEffect
}

data class PersonalVpnUiState(
    val enabled: Boolean,
    val clearEnabled: Boolean,
    val tunnelStats: TunnelStats,
    val initialFormData: PersonalVpnFormData = PersonalVpnFormData(),
    val privateKeyDataError: FormDataError.PrivateKey? = null,
    val tunnelIpDataError: FormDataError.TunnelIp? = null,
    val publicKeyDataError: FormDataError.PublicKey? = null,
    val allowedIpDataErrors: Map<Int, FormDataError.AllowedIp> = emptyMap(),
    val endpointDataError: FormDataError.Endpoint? = null,
)

data class PersonalVpnFormData(
    val privateKey: String = "",
    val tunnelIp: String = "",
    val publicKey: String = "",
    val allowedIPs: List<String> = listOf(""),
    val endpoint: String = "",
) {
    companion object {
        fun from(personalVpnConfig: PersonalVpnConfig?) =
            if (personalVpnConfig == null) {
                PersonalVpnFormData()
            } else {
                PersonalVpnFormData(
                    privateKey = personalVpnConfig.tunnelConfig.privateKey.value,
                    tunnelIp = personalVpnConfig.tunnelConfig.tunnelIp.hostAddress ?: "",
                    publicKey = personalVpnConfig.peerConfig.publicKey.value,
                    allowedIPs =
                        personalVpnConfig.peerConfig.allowedIps.ifEmpty { listOf("") },
                    endpoint =
                        "${personalVpnConfig.peerConfig.endpoint.hostString}:${personalVpnConfig.peerConfig.endpoint.port}",
                )
            }
    }
}

sealed interface FormDataError {
    data class PrivateKey(val keyParseError: KeyParseError) : FormDataError

    data object TunnelIp : FormDataError

    data class PublicKey(val keyParseError: KeyParseError) : FormDataError

    data class AllowedIp(val index: Int) : FormDataError

    sealed interface Endpoint : FormDataError {
        data object Empty : Endpoint

        data object InvalidAddress : Endpoint

        data class InvalidPort(val parsePortError: ParsePortError) : Endpoint
    }
}
