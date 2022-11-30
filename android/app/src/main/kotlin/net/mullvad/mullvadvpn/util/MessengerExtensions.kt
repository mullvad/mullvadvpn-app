package net.mullvad.mullvadvpn.util

import android.os.DeadObjectException
import android.os.Message
import android.os.Messenger
import android.util.Log

fun Messenger.trySend(message: Message, logErrors: Boolean): Boolean {
    return try {
        this.send(message)
        true
    } catch (ex: DeadObjectException) {
        if (logErrors) {
            Log.e("Mullvad", ex.toString())
        }
        false
    }
}
