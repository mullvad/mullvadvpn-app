package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import java.util.UUID
import kotlin.collections.ifEmpty
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.KeyParseError
import net.mullvad.mullvadvpn.lib.model.ParsePortError

data class CustomTunnelEditUiState(
    val formData: PersonalVpnFormData = PersonalVpnFormData(),
    val formErrors: PersonalVpnFormErrors? = null,
)

data class PersonalVpnFormErrors(
    val nameError: FormDataError.EmptyName? = null,
    val privateKeyError: FormDataError.PrivateKey? = null,
    val tunnelIpErrors: Map<String, FormDataError.TunnelIp> = emptyMap(),
    val peerErrors: Map<String, PeerErrors> = emptyMap(),
)

data class PeerErrors(
    val publicKeyError: FormDataError.PublicKey? = null,
    val allowedIpErrors: Map<String, FormDataError.AllowedIp> = emptyMap(),
    val endpointAddressError: FormDataError.EndpointAddress? = null,
    val endpointPortError: FormDataError.EndpointPort? = null,
)

data class PersonalVpnFormData(
    val enabled: Boolean = false,
    val name: String = "",
    val privateKey: String = "",
    val tunnelIps: Set<String> = setOf(""),
    val peers: List<PeerFormData> = listOf(PeerFormData()),
) {
    companion object {
        fun from(personalVpnConfig: CustomVpnConfig?) =
            if (personalVpnConfig == null) {
                PersonalVpnFormData()
            } else {
                PersonalVpnFormData(
                    name = personalVpnConfig.name.orEmpty(),
                    enabled = personalVpnConfig.enabled,
                    privateKey = personalVpnConfig.tunnelConfig.privateKey.value,
                    tunnelIps = personalVpnConfig.tunnelConfig.tunnelIps.ifEmpty { setOf("") },
                    peers =
                        personalVpnConfig.peerConfig.map { peerConfig ->
                            PeerFormData(
                                publicKey = peerConfig.publicKey.value,
                                allowedIps = peerConfig.allowedIps.ifEmpty { setOf("") },
                                endpointAddress = peerConfig.endpoint.hostString,
                                endpointPort = peerConfig.endpoint.port.toString(),
                            )
                        },
                )
            }
    }
}

data class PeerFormData(
    val id: String = UUID.randomUUID().toString(),
    val publicKey: String = "",
    val allowedIps: Set<String> = setOf(""),
    val endpointAddress: String = "",
    val endpointPort: String = "",
)

sealed interface FormDataError {
    data object EmptyName : FormDataError

    data class PrivateKey(val errorValue: String, val keyParseError: KeyParseError) : FormDataError

    data class TunnelIp(val errorValue: String) : FormDataError

    data class PublicKey(val errorValue: String, val keyParseError: KeyParseError) : FormDataError

    data class AllowedIp(val errorValue: String) : FormDataError

    data class EndpointAddress(val errorValue: String) : FormDataError

    data class EndpointPort(val errorValue: String, val parsePortError: ParsePortError) :
        FormDataError
}
