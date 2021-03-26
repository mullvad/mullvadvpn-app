package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import android.os.Messenger
import kotlinx.parcelize.Parcelize

// Requests that the service can handle
sealed class Request : Message() {
    protected override val messageId = 2
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    data class RegisterListener(val listener: Messenger) : Request()

    companion object {
        private const val MESSAGE_KEY = "request"

        fun fromMessage(message: RawMessage): Request? = Message.fromMessage(message, MESSAGE_KEY)
    }
}
