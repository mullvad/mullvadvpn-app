package net.mullvad.mullvadvpn.lib.ipc

import android.os.Message as RawMessage
import android.os.Messenger
import java.net.InetAddress
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.WireguardConstraints

// Requests that the service can handle
sealed class Request : Message.RequestMessage() {
    override val messageKey = MESSAGE_KEY

    @Parcelize
    @Deprecated("Use SetDnsOptions")
    data class AddCustomDnsServer(val address: InetAddress) : Request()

    @Parcelize object Connect : Request()

    @Parcelize object CreateAccount : Request()

    @Parcelize object Disconnect : Request()

    @Parcelize data class ExcludeApp(val packageName: String) : Request()

    @Parcelize object FetchAccountExpiry : Request()

    @Parcelize object FetchAccountHistory : Request()

    @Parcelize object FetchAuthToken : Request()

    @Parcelize data class IncludeApp(val packageName: String) : Request()

    @Parcelize data class Login(val account: String?) : Request()

    @Parcelize object RefreshDeviceState : Request()

    @Parcelize object GetDevice : Request()

    @Parcelize data class GetDeviceList(val accountToken: String) : Request()

    @Parcelize data class RemoveDevice(val accountToken: String, val deviceId: String) : Request()

    @Parcelize object Logout : Request()

    @Parcelize object PersistExcludedApps : Request()

    @Parcelize object Reconnect : Request()

    @Parcelize data class RegisterListener(val listener: Messenger) : Request()

    @Parcelize object ClearAccountHistory : Request()

    @Parcelize
    @Deprecated("Use SetDnsOptions")
    data class RemoveCustomDnsServer(val address: InetAddress) : Request()

    @Parcelize
    @Deprecated("Use SetDnsOptions")
    data class ReplaceCustomDnsServer(val oldAddress: InetAddress, val newAddress: InetAddress) :
        Request()

    @Parcelize data class SetAllowLan(val allow: Boolean) : Request()

    @Parcelize data class SetAutoConnect(val autoConnect: Boolean) : Request()

    @Parcelize
    @Deprecated("Use SetDnsOptions")
    data class SetEnableCustomDns(val enable: Boolean) : Request()

    @Parcelize data class SetEnableSplitTunneling(val enable: Boolean) : Request()

    @Parcelize
    data class SetRelayLocation(val relayLocation: GeographicLocationConstraint) : Request()

    @Parcelize data class SetWireGuardMtu(val mtu: Int?) : Request()

    @Parcelize data class SubmitVoucher(val voucher: String) : Request()

    @Parcelize data class UnregisterListener(val listenerId: Int) : Request()

    @Parcelize data class VpnPermissionResponse(val isGranted: Boolean) : Request()

    @Parcelize data class SetDnsOptions(val dnsOptions: DnsOptions) : Request()

    @Parcelize data class SetObfuscationSettings(val settings: ObfuscationSettings?) : Request()

    @Parcelize
    data class SetWireguardConstraints(val wireguardConstraints: WireguardConstraints) : Request()

    @Parcelize
    data class SetWireGuardQuantumResistant(val quantumResistant: QuantumResistantState) :
        Request()

    companion object {
        private const val MESSAGE_KEY = "request"

        fun fromMessage(message: RawMessage): Request? = fromMessage(message, MESSAGE_KEY)
    }
}
