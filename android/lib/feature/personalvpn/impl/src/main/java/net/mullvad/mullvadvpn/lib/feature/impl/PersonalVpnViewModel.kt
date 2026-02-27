package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.compose.runtime.Composable
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.Either.Companion.zipOrAccumulate
import arrow.core.EitherNel
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
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.common.util.onFirst
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats
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
            managementService.customVpnStats(),
                formData.filterNotNull(),
                formErrors,
            ) { customVpnEnabled: Boolean, customVpnStats: TunnelStats, formData: PersonalVpnFormData, formErrors ->
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

    private fun parsePrivateKey(privateKey: String): Either<FormDataError.PrivateKey, String> {
        return Either.Right(privateKey)
    }

    private fun parseAddress(address: String): Either<FormDataError.TunnelIp, InetAddress> =
        Either.catch { InetAddress.getByName(address) }.mapLeft { FormDataError.TunnelIp }

    private fun parsePublicKey(privateKey: String): Either<FormDataError.PublicKey, String> {
        return Either.Right(privateKey)
    }

    private fun parseAllowedIp(privateKey: String): Either<FormDataError.AllowedIp, String> {
        return Either.Right(privateKey)
    }

    private fun parseEndpoint(endpoint: String): Either<FormDataError.Endpoint, InetSocketAddress> =
        Either.catch {
                val (host, port) = endpoint.trim().split(':')
                val parsedHost = InetAddress.getByName(host)
                val parsedPort = port.toInt()
                InetSocketAddress(parsedHost, parsedPort)
            }
            .mapLeft { FormDataError.Endpoint }
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
                    privateKey = customVpnConfig.tunnelConfig.privateKey,
                    tunnelIp = customVpnConfig.tunnelConfig.tunnelIp.hostAddress ?: "",
                    publicKey = customVpnConfig.peerConfig.publicKey,
                    allowedIP = customVpnConfig.peerConfig.allowedIp,
                    endpoint = customVpnConfig.peerConfig.endpoint.hostString,
                )
            }
    }
}

sealed interface FormDataError {
    data object PrivateKey : FormDataError

    data object TunnelIp : FormDataError

    data object PublicKey : FormDataError

    data object AllowedIp : FormDataError

    data object Endpoint : FormDataError
}

@Composable
fun FormDataError.toErrorMessage(): String =
    when (this) {
        FormDataError.AllowedIp -> "Bad allowed IP"
        FormDataError.Endpoint -> "Bad endpoint"
        FormDataError.PrivateKey -> "Bad private key"
        FormDataError.PublicKey -> "Bad public key"
        FormDataError.TunnelIp -> "Bad address IP"
    }
