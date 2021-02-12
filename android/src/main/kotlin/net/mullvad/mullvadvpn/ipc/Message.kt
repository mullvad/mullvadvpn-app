package net.mullvad.mullvadvpn.ipc

import android.os.Bundle
import android.os.Message as RawMessage
import android.os.Parcelable

abstract class Message : Parcelable {
    protected abstract val messageId: Int
    protected abstract val messageKey: String

    val message: RawMessage
        get() = RawMessage.obtain().also { message ->
            message.what = messageId
            message.data = Bundle()
            message.data.putParcelable(messageKey, this)
        }

    companion object {
        internal fun <T : Parcelable> fromMessage(message: RawMessage, key: String): T? {
            val data = message.data

            return data.getParcelable(key)
        }
    }
}
