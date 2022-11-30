package net.mullvad.mullvadvpn.util

import android.os.DeadObjectException
import android.os.Message
import android.os.Messenger
import android.os.RemoteException
import android.util.Log

fun Messenger.trySend(message: Message, logErrors: Boolean): Boolean {
    return try {
        this.send(message)
        true
    } catch (deadObjectException: DeadObjectException) {
        if (logErrors) {
            Log.e("mullvad", deadObjectException.toString())
        }
        false
    } catch (remoteException: RemoteException) {
        if (logErrors) {
            Log.e("mullvad", remoteException.toString())
        }
        false
    }
}
