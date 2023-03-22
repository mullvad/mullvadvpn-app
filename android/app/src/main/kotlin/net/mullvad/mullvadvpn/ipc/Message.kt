package net.mullvad.mullvadvpn.ipc

import android.os.Bundle
import android.os.Message as RawMessage
import android.os.Parcelable

sealed class Message(private val messageId: Int) : Parcelable {
    abstract class EventMessage : Message(1)
    abstract class RequestMessage : Message(2)

    protected abstract val messageKey: String

    val message: RawMessage
        get() =
            RawMessage.obtain().also { message ->
                message.what = messageId
                message.data = Bundle()
                message.data.putParcelable(messageKey, this)
            }

    companion object {
        internal fun <T : Parcelable> fromMessage(message: RawMessage, key: String): T? {
            val data = message.data

            data.classLoader = Message::class.java.classLoader

            return data.getParcelable(key)
        }
    }
}
