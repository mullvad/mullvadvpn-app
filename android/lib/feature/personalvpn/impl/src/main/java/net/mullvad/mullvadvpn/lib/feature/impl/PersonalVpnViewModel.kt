package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.Either.Companion.zipOrAccumulate
import arrow.core.EitherNel
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
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.common.util.onFirst
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
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
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val _uiSideEffect = Channel<PersonalVpnSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    private val formData = MutableStateFlow<PersonalVpnFormData?>(null)
    private val formErrors = MutableStateFlow<List<FormDataError>>(emptyList())

    val uiState: StateFlow<Lc<Boolean, PersonalVpnUiState>> =
        combine(
                settingsRepository.settingsUpdates
                    .filterNotNull()
                    .onFirst { formData.value = PersonalVpnFormData.from(it.customVpnConfig) }
                    .map { it.customVpnEnabled },
                managementService.tunnelStats.onStart { emit(TunnelStats()) },
                formData.filterNotNull(),
                formErrors,
            ) {
                customVpnEnabled: Boolean,
                customVpnStats: TunnelStats,
                formData: PersonalVpnFormData,
                formErrors ->
                PersonalVpnUiState(
                        enabled = customVpnEnabled,
                        tunnelStats = customVpnStats,
                        formData,
                        formErrors.filterIsInstance<FormDataError.PrivateKey>().firstOrNull(),
                        formErrors.filterIsInstance<FormDataError.TunnelIp>().firstOrNull(),
                        formErrors.filterIsInstance<FormDataError.PublicKey>().firstOrNull(),
                        formErrors.filterIsInstance<FormDataError.AllowedIp>().firstOrNull(),
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
        viewModelScope.launch { managementService.toggleCustomVpn(on) }
    }

    fun save(formData: PersonalVpnFormData) {
        parseFormData(formData)
            .fold(
                { formErrors.value = it },
                {
                    formErrors.value = emptyList()
                    viewModelScope.launch {
                        managementService
                            .setCustomVpnConfig(it)
                            .fold(
                                {
                                    _uiSideEffect.send(
                                        PersonalVpnSideEffect.FailedToSave(it.toString())
                                    )
                                },
                                {},
                            )
                    }
                },
            )
    }

    private fun parseFormData(
        formData: PersonalVpnFormData
    ): EitherNel<FormDataError, CustomVpnConfig> =
        zipOrAccumulate(
            parsePrivateKey(formData.privateKey),
            parseAddress(formData.tunnelIp),
            parsePublicKey(formData.publicKey),
            parseAllowedIp(formData.allowedIP),
            parseEndpoint(formData.endpoint),
        ) { privateKey, tunnelIp, publicKey, allowedIp, endpoint ->
            CustomVpnConfig(
                tunnelConfig = TunnelConfig(privateKey = privateKey, tunnelIp = tunnelIp),
                peerConfig =
                    PeerConfig(publicKey = publicKey, allowedIp = allowedIp, endpoint = endpoint),
            )
        }

    private fun parsePrivateKey(
        privateKey: String
    ): Either<FormDataError.PrivateKey, WireguardKey> {
        return WireguardKey.from(privateKey).mapLeft { FormDataError.PrivateKey(it) }
    }

    private fun parseAddress(address: String): Either<FormDataError.TunnelIp, InetAddress> =
        Either.catch { InetAddress.getByName(address) }.mapLeft { FormDataError.TunnelIp }

    private fun parsePublicKey(privateKey: String): Either<FormDataError.PublicKey, WireguardKey> {
        return WireguardKey.from(privateKey).mapLeft { FormDataError.PublicKey(it) }
    }

    private fun parseAllowedIp(privateKey: String): Either<FormDataError.AllowedIp, String> {
        return Either.Right(privateKey)
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
    data class FailedToSave(val reason: String) : PersonalVpnSideEffect
}

data class PersonalVpnUiState(
    val enabled: Boolean,
    val tunnelStats: TunnelStats,
    val initialFormData: PersonalVpnFormData = PersonalVpnFormData(),
    val privateKeyDataError: FormDataError.PrivateKey? = null,
    val tunnelIpDataError: FormDataError.TunnelIp? = null,
    val publicKeyDataError: FormDataError.PublicKey? = null,
    val allowedIpDataError: FormDataError.AllowedIp? = null,
    val endpointDataError: FormDataError.Endpoint? = null,
)

data class PersonalVpnFormData(
    val privateKey: String = "",
    val tunnelIp: String = "",
    val publicKey: String = "",
    val allowedIP: String = "",
    val endpoint: String = "",
) {
    companion object {
        fun from(customVpnConfig: CustomVpnConfig?) =
            if (customVpnConfig == null) {
                PersonalVpnFormData()
            } else {
                PersonalVpnFormData(
                    privateKey = customVpnConfig.tunnelConfig.privateKey.value,
                    tunnelIp = customVpnConfig.tunnelConfig.tunnelIp.hostAddress ?: "",
                    publicKey = customVpnConfig.peerConfig.publicKey.value,
                    allowedIP = customVpnConfig.peerConfig.allowedIp,
                    endpoint = customVpnConfig.peerConfig.endpoint.hostString,
                )
            }
    }
}

sealed interface FormDataError {
    data class PrivateKey(val keyParseError: KeyParseError) : FormDataError

    data object TunnelIp : FormDataError

    data class PublicKey(val keyParseError: KeyParseError) : FormDataError

    data object AllowedIp : FormDataError

    sealed interface Endpoint : FormDataError {
        data object Empty : Endpoint

        data object InvalidAddress : Endpoint

        data class InvalidPort(val parsePortError: ParsePortError) : Endpoint
    }
}
