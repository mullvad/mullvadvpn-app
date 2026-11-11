package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import java.net.InetSocketAddress
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.WireguardKey
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

class CustomTunnelEditViewModel(
    settingsRepository: SettingsRepository,
    private val managementService: ManagementService,
) : ViewModel() {
    private val _uiSideEffect = Channel<PersonalVpnSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    private val formData = MutableStateFlow<PersonalVpnFormData?>(null)
    private val formErrors = MutableStateFlow<PersonalVpnFormErrors?>(null)

    val uiState: StateFlow<Lc<Boolean, CustomTunnelEditUiState>> =
        combine(
                formData.filterNotNull().onStart {
                    val settings = settingsRepository.settingsUpdates.filterNotNull().first()
                    emit(PersonalVpnFormData.from(settings.customVpnConfig))
                },
                formErrors,
            ) { formData: PersonalVpnFormData, formErrors ->
                CustomTunnelEditUiState(
                        formData = formData,
                        formErrors = formErrors,
                    )
                    .toLc<Boolean, CustomTunnelEditUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(false),
            )

    fun clearConfig() {
        viewModelScope.launch {
            managementService.clearPersonalVpn()
            formData.value = PersonalVpnFormData.from(null)
        }
    }

    fun save(form: PersonalVpnFormData) = viewModelScope.launch {
        parseFormData(form)
            .fold(
                {
                    formErrors.value = it
                    _uiSideEffect.send(PersonalVpnSideEffect.FormErrors)
                },
                { vpnConfig ->
                    formErrors.value = null
                    managementService
                        .setPersonalVpnConfig(vpnConfig)
                        .fold(
                            {
                                _uiSideEffect.send(
                                    PersonalVpnSideEffect.FailedToSave(it.toString())
                                )
                            },
                            {
                                formData.value = PersonalVpnFormData.from(vpnConfig)
                                _uiSideEffect.send(PersonalVpnSideEffect.ConfigurationSaved)
                            },
                        )
                },
            )
    }

    private fun parseFormData(
        formData: PersonalVpnFormData
    ): Either<PersonalVpnFormErrors, CustomVpnConfig> {
        val nameError = validateName(formData.name)
        val (privateKeyError, privateKey) = parsePrivateKey(formData.privateKey)
        val (tunnelIpErrors, tunnelIps) = parseTunnelIps(formData.tunnelIps)
        val (peerErrors, peerConfigs) = parsePeers(formData.peers)

        return if (
            nameError == null && privateKey != null && tunnelIps != null && peerConfigs != null
        ) {
            Either.Right(
                CustomVpnConfig(
                    tunnelConfig = TunnelConfig(privateKey = privateKey, tunnelIps = tunnelIps),
                    peerConfig = peerConfigs,
                    enabled = formData.enabled,
                    name = formData.name,
                )
            )
        } else {
            Either.Left(
                PersonalVpnFormErrors(
                    nameError = nameError,
                    privateKeyError = privateKeyError,
                    tunnelIpErrors = tunnelIpErrors.orEmpty(),
                    peerErrors = peerErrors.orEmpty(),
                )
            )
        }
    }

    private fun validateName(name: String): FormDataError.EmptyName? =
        if (name.isEmpty()) {
            FormDataError.EmptyName
        } else {
            null
        }

    private fun parsePeers(
        peers: List<PeerFormData>
    ): Either<Map<String, PeerErrors>, List<PeerConfig>> {
        val errors = mutableMapOf<String, PeerErrors>()

        val parsed = peers.mapNotNull { peer ->
            parsePeer(peer)
                .fold(
                    { error ->
                        errors[peer.id] = error
                        null
                    },
                    { config -> config },
                )
        }

        return if (errors.isNotEmpty()) {
            Either.Left(errors)
        } else {
            Either.Right(parsed)
        }
    }

    private fun parsePeer(peer: PeerFormData): Either<PeerErrors, PeerConfig> {
        val (publicKeyError, publicKey) = parsePeerPublicKey(peer.publicKey)
        val (allowedIpsError, allowedIps) = parseAllowedIpsCidr(peer.allowedIps)
        val (endpointAddressError, endpointAddress) = parseEndpointAddress(peer.endpointAddress)
        val (endpointPortError, endpointPort) = parseEndpointPort(peer.endpointPort)

        return if (
            publicKey != null &&
                allowedIps != null &&
                endpointAddress != null &&
                endpointPort != null
        ) {
            Either.Right(
                PeerConfig(
                    publicKey = publicKey,
                    endpoint = InetSocketAddress(endpointAddress, endpointPort),
                    allowedIps = allowedIps,
                )
            )
        } else {
            Either.Left(
                PeerErrors(
                    publicKeyError = publicKeyError,
                    allowedIpErrors = allowedIpsError.orEmpty(),
                    endpointAddressError = endpointAddressError,
                    endpointPortError = endpointPortError,
                )
            )
        }
    }

    private fun parsePrivateKey(
        privateKey: String
    ): Either<FormDataError.PrivateKey, WireguardKey> {
        return WireguardKey.from(privateKey).mapLeft {
            FormDataError.PrivateKey(errorValue = privateKey, keyParseError = it)
        }
    }

    private fun parsePeerPublicKey(
        publicKey: String
    ): Either<FormDataError.PublicKey, WireguardKey> {
        return WireguardKey.from(publicKey).mapLeft {
            FormDataError.PublicKey(errorValue = publicKey, keyParseError = it)
        }
    }

    private fun parseTunnelIps(
        tunnelIps: Set<String>
    ): Either<Map<String, FormDataError.TunnelIp>, Set<String>> {
        val (valid, invalid) = tunnelIps.partition(::isValidIp)

        return if (invalid.isEmpty()) {
            Either.Right(valid.toSet())
        } else {
            Either.Left(invalid.associateWith(FormDataError::TunnelIp))
        }
    }

    private fun parseAllowedIpsCidr(
        allowedIps: Set<String>
    ): Either<Map<String, FormDataError.AllowedIp>, Set<String>> {
        val (valid, invalid) = allowedIps.partition(::isValidIpOrCidr)

        return if (invalid.isEmpty()) {
            Either.Right(valid.toSet())
        } else {
            Either.Left(invalid.associateWith(FormDataError::AllowedIp))
        }
    }

    private fun parseEndpointAddress(
        endpointAddress: String
    ): Either<FormDataError.EndpointAddress, String> =
        if (isValidIp(endpointAddress)) {
            Either.Right(endpointAddress)
        } else {
            Either.Left(FormDataError.EndpointAddress(endpointAddress))
        }

    private fun parseEndpointPort(endpointPort: String): Either<FormDataError.EndpointPort, Int> =
        when (val port = endpointPort.toIntOrNull()) {
            null ->
                Either.Left(
                    FormDataError.EndpointPort(
                        errorValue = endpointPort,
                        ParsePortError.NotANumber(endpointPort),
                    )
                )
            !in 1..Port.MAX_VALUE ->
                Either.Left(
                    FormDataError.EndpointPort(
                        errorValue = endpointPort,
                        ParsePortError.OutOfRange(port),
                    )
                )
            else -> Either.Right(port)
        }
}

operator fun <L, R> Either<L, R>.component1(): L? = leftOrNull()

operator fun <L, R> Either<L, R>.component2(): R? = getOrNull()

private const val IPV4_OCTET = """(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]\d|\d)"""
private const val IPV4_BASE = """$IPV4_OCTET(\.$IPV4_OCTET){3}"""

private const val IPV6_BASE =
    "(" +
        "([0-9a-f]{1,4}:){7}[0-9a-f]{1,4}|" + // 1:2:3:4:5:6:7:8
        "([0-9a-f]{1,4}:){1,7}:|" + // 1:: to 1:2:3:4:5:6:7::
        "([0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}|" + // 1::8 to 1:2:3:4:5:6::8
        "([0-9a-f]{1,4}:){1,5}(:[0-9a-f]{1,4}){1,2}|" +
        "([0-9a-f]{1,4}:){1,4}(:[0-9a-f]{1,4}){1,3}|" +
        "([0-9a-f]{1,4}:){1,3}(:[0-9a-f]{1,4}){1,4}|" +
        "([0-9a-f]{1,4}:){1,2}(:[0-9a-f]{1,4}){1,5}|" +
        "[0-9a-f]{1,4}:((:[0-9a-f]{1,4}){1,6})|" + // 1::2:3:4:5:6:7
        ":((:[0-9a-f]{1,4}){1,7}|:)" + // ::2:3:4:5:6:7 or ::
        ")"

private val IPV4_EXACT_REGEX = """^$IPV4_BASE$""".toRegex()
private val IPV6_EXACT_REGEX = """^$IPV6_BASE$""".toRegex(RegexOption.IGNORE_CASE)

// CIDR Regexes (Allows base IP, plus optional /0 to /32 or /128)
private val IPV4_CIDR_REGEX = """^$IPV4_BASE(/([0-9]|[12][0-9]|3[0-2]))?$""".toRegex()
private val IPV6_CIDR_REGEX =
    """^$IPV6_BASE(/([0-9]|[1-9][0-9]|1[01][0-9]|12[0-8]))?$""".toRegex(RegexOption.IGNORE_CASE)

private fun isValidIpOrCidr(raw: String): Boolean =
    IPV4_CIDR_REGEX.matches(raw) || IPV6_CIDR_REGEX.matches(raw)

private fun isValidIp(raw: String): Boolean =
    IPV4_EXACT_REGEX.matches(raw) || IPV6_EXACT_REGEX.matches(raw)
