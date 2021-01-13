package net.mullvad.mullvadvpn.service

import android.os.Bundle
import android.os.Message
import android.os.Messenger
import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.model.LocationConstraint
import org.joda.time.DateTime

sealed class Request : Parcelable {
    @Parcelize
    class AddCustomDnsServer(val address: InetAddress) : Request(), Parcelable

    @Parcelize
    class Connect : Request(), Parcelable

    @Parcelize
    class CreateAccount : Request(), Parcelable

    @Parcelize
    class Disconnect : Request(), Parcelable

    @Parcelize
    class ExcludeApp(val packageName: String?) : Request(), Parcelable

    @Parcelize
    class FetchAccountExpiry : Request(), Parcelable

    @Parcelize
    class FetchAuthToken : Request(), Parcelable

    @Parcelize
    class IncludeApp(val packageName: String?) : Request(), Parcelable

    @Parcelize
    class InvalidateAccountExpiry(val expiry: DateTime) : Request(), Parcelable

    @Parcelize
    class Login(val account: String?) : Request(), Parcelable

    @Parcelize
    class Logout : Request(), Parcelable

    @Parcelize
    class PersistExcludedApps : Request(), Parcelable

    @Parcelize
    class Reconnect : Request(), Parcelable

    @Parcelize
    class RegisterListener(val listener: Messenger) : Request(), Parcelable

    @Parcelize
    class RemoveAccountFromHistory(val account: String?) : Request(), Parcelable

    @Parcelize
    class RemoveCustomDnsServer(val address: InetAddress) : Request(), Parcelable

    @Parcelize
    class ReplaceCustomDnsServer(
        val oldAddress: InetAddress,
        val newAddress: InetAddress
    ) : Request(), Parcelable

    @Parcelize
    class SetAccount(val account: String?) : Request(), Parcelable

    @Parcelize
    class SetAllowLan(val allow: Boolean) : Request(), Parcelable

    @Parcelize
    class SetAutoConnect(val autoConnect: Boolean) : Request(), Parcelable

    @Parcelize
    class SetEnableCustomDns(val enable: Boolean) : Request(), Parcelable

    @Parcelize
    class SetEnableSplitTunneling(val enable: Boolean) : Request(), Parcelable

    @Parcelize
    class SetRelayLocation(val relayLocation: LocationConstraint?) : Request(), Parcelable

    @Parcelize
    class SetWireGuardMtu(val mtu: Int?) : Request(), Parcelable

    @Parcelize
    class VpnPermissionResponse(val vpnPermission: Boolean) : Request(), Parcelable

    @Parcelize
    class WireGuardGenerateKey : Request(), Parcelable

    @Parcelize
    class WireGuardVerifyKey : Request(), Parcelable

    val message: Message
        get() = Message.obtain().also { message ->
            message.what = REQUEST_MESSAGE
            message.data = Bundle()
            message.data.putParcelable(REQUEST_KEY, this)
        }

    companion object {
        const val REQUEST_MESSAGE = 2
        const val REQUEST_KEY = "request"

        fun fromMessage(message: Message): Request {
            val data = message.data

            return data.getParcelable(REQUEST_KEY)!!
        }
    }
}
