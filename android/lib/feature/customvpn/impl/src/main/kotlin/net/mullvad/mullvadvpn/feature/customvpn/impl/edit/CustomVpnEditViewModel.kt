package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.EitherNel
import arrow.core.flatMap
import arrow.core.mapOrAccumulate
import arrow.core.merge
import arrow.core.raise.context.ensure
import arrow.core.raise.either
import java.net.InetSocketAddress
import kotlin.collections.associateBy
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.common.util.zipFold
import net.mullvad.mullvadvpn.lib.common.validator.IpValidator
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.WireguardKey
import net.mullvad.mullvadvpn.lib.repository.CustomVpnRepository

class CustomVpnEditViewModel(
    private val customVpnRepository: CustomVpnRepository,
    private val ipValidator: IpValidator,
    isNewConfig: Boolean,
) : ViewModel() {
    private val _uiSideEffect = Channel<CustomVpnSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val formData = MutableStateFlow(FormData())
    private val formErrors = MutableStateFlow<FormErrors?>(null)
    private val isNew = MutableStateFlow(isNewConfig)

    val uiState: StateFlow<Lc<Unit, CustomVpnEditUiState>> =
        combine(
                customVpnRepository.configFlow.map { config ->
                    config?.let { FormData.from(it) } ?: FormData()
                },
                formErrors,
                isNew,
            ) { formData, formErrors, isNew ->
                CustomVpnEditUiState(
                        formData = formData,
                        formErrors = formErrors,
                        isNewConfig = isNew,
                    )
                    .toLc<Unit, CustomVpnEditUiState>()
            }
            .onStart { customVpnRepository.fetchConfig() }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun clearConfig() {
        viewModelScope.launch {
            customVpnRepository.clearConfig()
            formData.value = FormData()
            isNew.value = false
        }
    }

    fun save(form: FormData) = viewModelScope.launch {
        val sideEffect: CustomVpnSideEffect =
            parseFormData(form)
                .onRight { formErrors.value = null }
                .mapLeft {
                    formErrors.value = it
                    CustomVpnSideEffect.FormErrors
                }
                .flatMap { vpnConfig ->
                    customVpnRepository
                        .setConfig(vpnConfig)
                        .onRight { formData.value = FormData.from(vpnConfig) }
                        .mapLeft { CustomVpnSideEffect.FailedToSave(it.toString()) }
                        .map { CustomVpnSideEffect.ConfigurationSaved }
                }
                .merge()

        _uiSideEffect.send(sideEffect)
    }

    private fun parseFormData(formData: FormData): Either<FormErrors, CustomVpnConfig> =
        zipFold(
            validateName(formData.name),
            parsePrivateKey(formData.privateKey),
            parseTunnelIps(formData.tunnelIps),
            parsePeers(formData.peers),
            transformError = { nameError, privateKeyError, tunnelIpError, parsePeersError ->
                FormErrors(
                    nameError = nameError,
                    privateKeyError = privateKeyError,
                    tunnelIpErrors = tunnelIpError?.associateBy { it.errorValue }.orEmpty(),
                    peerErrors = parsePeersError?.toMap().orEmpty(),
                )
            },
        ) { name, privateKey, tunnelIps, peerConfigs ->
            CustomVpnConfig(
                tunnelConfig = TunnelConfig(privateKey = privateKey, tunnelIps = tunnelIps),
                peerConfig = peerConfigs,
                enabled = formData.enabled,
                name = name,
            )
        }

    private fun validateName(name: String): Either<FormDataError.EmptyName, String> = either {
        ensure(name.isNotEmpty()) { FormDataError.EmptyName }
        name
    }

    private fun parsePeers(
        peers: List<PeerFormData>,
    ): EitherNel<Pair<String, PeerErrors>, List<PeerConfig>> = peers.mapOrAccumulate {
        parsePeer(it).mapLeft { peerErrors -> it.id to peerErrors }.bind()
    }

    private fun parsePeer(peer: PeerFormData): Either<PeerErrors, PeerConfig> =
        zipFold(
            parsePeerPublicKey(peer.publicKey),
            parseAllowedIpsCidr(peer.allowedIps),
            parseEndpointAddress(peer.endpointAddress),
            parseEndpointPort(peer.endpointPort),
            { publicKeyError, allowedIpsError, endpointAddressError, endpointPortError ->
                PeerErrors(
                    publicKeyError = publicKeyError,
                    allowedIpErrors = allowedIpsError.orEmpty(),
                    endpointAddressError = endpointAddressError,
                    endpointPortError = endpointPortError,
                )
            },
        ) {
            publicKey: WireguardKey,
            allowedIps: Set<String>,
            endpointAddress: String,
            endpointPort: Port ->
            PeerConfig(
                publicKey = publicKey,
                endpoint = InetSocketAddress(endpointAddress, endpointPort.value),
                allowedIps = allowedIps,
            )
        }

    private fun parsePrivateKey(
        privateKey: String,
    ): Either<FormDataError.PrivateKey, WireguardKey> =
        WireguardKey.from(privateKey).mapLeft {
            FormDataError.PrivateKey(errorValue = privateKey, keyParseError = it)
        }

    private fun parsePeerPublicKey(
        publicKey: String,
    ): Either<FormDataError.PublicKey, WireguardKey> =
        WireguardKey.from(publicKey).mapLeft {
            FormDataError.PublicKey(errorValue = publicKey, keyParseError = it)
        }

    private fun parseTunnelIps(
        tunnelIps: Set<String>,
    ): EitherNel<FormDataError.TunnelIp, Set<String>> =
        tunnelIps
            .mapOrAccumulate {
                ensure(ipValidator.isValidIp(it)) { FormDataError.TunnelIp(it) }
                it
            }
            .map { it.toSet() }

    private fun parseAllowedIpsCidr(
        allowedIps: Set<String>,
    ): Either<Map<String, FormDataError.AllowedIp>, Set<String>> =
        allowedIps
            .mapOrAccumulate { ipAddress ->
                ensure(ipValidator.isValidIpOrCidr(ipAddress)) {
                    ipAddress to FormDataError.AllowedIp(ipAddress)
                }
                ipAddress
            }
            .mapLeft { it.toMap() }
            .map { it.toSet() }

    private fun parseEndpointAddress(
        endpointAddress: String,
    ): Either<FormDataError.EndpointAddress, String> = either {
        ensure(ipValidator.isValidIp(endpointAddress)) {
            FormDataError.EndpointAddress(endpointAddress)
        }
        endpointAddress
    }

    private fun parseEndpointPort(endpointPort: String): Either<FormDataError.EndpointPort, Port> =
        Port.fromString(endpointPort).mapLeft {
            FormDataError.EndpointPort(errorValue = endpointPort, parsePortError = it)
        }
}
