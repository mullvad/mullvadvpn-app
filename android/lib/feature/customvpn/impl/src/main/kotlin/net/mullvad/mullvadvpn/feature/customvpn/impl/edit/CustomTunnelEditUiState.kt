package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import java.util.UUID
import kotlin.collections.ifEmpty
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.KeyParseError
import net.mullvad.mullvadvpn.lib.model.ParsePortError

data class CustomTunnelEditUiState(
    val formData: FormData = FormData(),
    val formErrors: FormErrors? = null,
)

data class FormErrors(
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

data class FormData(
    val enabled: Boolean = false,
    val name: String = "",
    val privateKey: String = "",
    val tunnelIps: Set<String> = setOf(""),
    val peers: List<PeerFormData> = listOf(PeerFormData()),
) {
    companion object {
        fun from(customVpnConfig: CustomVpnConfig?) =
            if (customVpnConfig == null) {
                FormData()
            } else {
                FormData(
                    name = customVpnConfig.name.orEmpty(),
                    enabled = customVpnConfig.enabled,
                    privateKey = customVpnConfig.tunnelConfig.privateKey.value,
                    tunnelIps = customVpnConfig.tunnelConfig.tunnelIps.ifEmpty { setOf("") },
                    peers =
                        customVpnConfig.peerConfig.map { peerConfig ->
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
