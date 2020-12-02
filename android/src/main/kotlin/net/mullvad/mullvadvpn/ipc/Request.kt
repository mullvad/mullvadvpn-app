package net.mullvad.mullvadvpn.ipc

import android.os.Message as RawMessage
import android.os.Messenger
import android.os.Parcelable
import kotlinx.parcelize.Parcelize

// Requests that the service can handle
sealed class Request : Message(), Parcelable {
    protected override val messageId = 2
    protected override val messageKey = MESSAGE_KEY

    @Parcelize
    data class RegisterListener(val listener: Messenger) : Request(), Parcelable

    companion object {
        private const val MESSAGE_KEY = "request"

        fun fromMessage(message: RawMessage): Request? = Message.fromMessage(message, MESSAGE_KEY)
    }
}
