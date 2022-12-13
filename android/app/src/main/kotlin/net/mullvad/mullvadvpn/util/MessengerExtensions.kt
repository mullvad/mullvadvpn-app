package net.mullvad.mullvadvpn.util

import android.os.DeadObjectException
import android.os.Message
import android.os.Messenger
import android.os.RemoteException
import android.util.Log
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request

fun Messenger.trySendEvent(event: Event, logErrors: Boolean): Boolean {
    return trySend(event.message, logErrors, event::class.qualifiedName)
}

fun Messenger.trySendRequest(request: Request, logErrors: Boolean): Boolean {
    return trySend(request.message, logErrors, request::class.qualifiedName)
}

private fun Messenger.trySend(message: Message, logErrors: Boolean, messageName: String?): Boolean {
    return try {
        this.send(message)
        true
    } catch (deadObjectException: DeadObjectException) {
        if (logErrors) {
            Log.e(
                "mullvad",
                "Failed to send message ${messageName ?: "<missing>"} due to DeadObjectException"
            )
        }
        false
    } catch (remoteException: RemoteException) {
        if (logErrors) {
            Log.e(
                "mullvad",
                "Failed to send message ${messageName ?: "<missing>"} due to RemoteException"
            )
        }
        false
    }
}
