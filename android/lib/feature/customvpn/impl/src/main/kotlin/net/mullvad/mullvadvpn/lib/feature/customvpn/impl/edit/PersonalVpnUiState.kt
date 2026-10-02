package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.edit

import kotlin.collections.ifEmpty
import net.mullvad.mullvadvpn.lib.model.KeyParseError
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.PersonalVpnConfig

data class PersonalVpnUiState(
    val clearEnabled: Boolean,
    val initialFormData: PersonalVpnFormData = PersonalVpnFormData(),
    val nameDataError: FormDataError.Name? = null,
    val privateKeyDataError: FormDataError.PrivateKey? = null,
    val tunnelIpDataErrors: Map<Int, FormDataError.TunnelIp> = emptyMap(),
    val publicKeyDataError: FormDataError.PublicKey? = null,
    val allowedIpDataErrors: Map<Int, FormDataError.AllowedIp> = emptyMap(),
    val endpointAddressDataError: FormDataError.EndpointAddress? = null,
    val endpointPortDataError: FormDataError.EndpointPort? = null,
)

data class PersonalVpnFormData(
    val name: String = "",
    val privateKey: String = "",
    val tunnelIps: List<String> = listOf(""),
    val publicKey: String = "",
    val allowedIPs: List<String> = listOf(""),
    val endpointAddress: String = "",
    val endpointPort: Int? = null,
) {
    companion object {
        fun from(personalVpnConfig: PersonalVpnConfig?) =
            if (personalVpnConfig == null) {
                PersonalVpnFormData()
            } else {
                PersonalVpnFormData(
                    name = "CustomVPN", // TODO
                    privateKey = personalVpnConfig.tunnelConfig.privateKey.value,
                    tunnelIps =
                        personalVpnConfig.tunnelConfig.tunnelIps
                            .map { it.hostAddress ?: "" }
                            .ifEmpty { listOf("") },
                    publicKey = personalVpnConfig.peerConfig.publicKey.value,
                    allowedIPs = personalVpnConfig.peerConfig.allowedIps.ifEmpty { listOf("") },
                    endpointAddress = personalVpnConfig.peerConfig.endpoint.hostString,
                    endpointPort = personalVpnConfig.peerConfig.endpoint.port,
                )
            }
    }
}

sealed interface FormDataError {
    data object Name : FormDataError // Empty

    data class PrivateKey(val keyParseError: KeyParseError) : FormDataError

    data class TunnelIp(val index: Int) : FormDataError

    data class PublicKey(val keyParseError: KeyParseError) : FormDataError

    data class AllowedIp(val index: Int) : FormDataError

    sealed interface EndpointAddress : FormDataError {
        data object Empty : EndpointAddress

        data object InvalidAddress : EndpointAddress
    }

    data class EndpointPort(val parsePortError: ParsePortError?) : FormDataError
}
