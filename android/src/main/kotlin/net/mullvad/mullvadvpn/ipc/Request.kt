package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import android.os.Messenger
import kotlinx.parcelize.Parcelize
import org.joda.time.DateTime

// Requests that the service can handle
sealed class Request : Message.RequestMessage() {
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    object Connect : Request()

    @Parcelize
    object CreateAccount : Request()

    @Parcelize
    object Disconnect : Request()

    @Parcelize
    data class ExcludeApp(val packageName: String) : Request()

    @Parcelize
    object FetchAccountExpiry : Request()

    @Parcelize
    data class IncludeApp(val packageName: String) : Request()

    @Parcelize
    data class InvalidateAccountExpiry(val expiry: DateTime) : Request()

    @Parcelize
    data class Login(val account: String?) : Request()

    @Parcelize
    object Logout : Request()

    @Parcelize
    object PersistExcludedApps : Request()

    @Parcelize
    object Reconnect : Request()

    @Parcelize
    data class RegisterListener(val listener: Messenger) : Request()

    @Parcelize
    data class RemoveAccountFromHistory(val account: String?) : Request()

    @Parcelize
    data class SetEnableSplitTunneling(val enable: Boolean) : Request()

    @Parcelize
    object WireGuardGenerateKey : Request()

    @Parcelize
    object WireGuardVerifyKey : Request()

    companion object {
        private const val MESSAGE_KEY = "request"

        fun fromMessage(message: RawMessage): Request? = Message.fromMessage(message, MESSAGE_KEY)
    }
}
