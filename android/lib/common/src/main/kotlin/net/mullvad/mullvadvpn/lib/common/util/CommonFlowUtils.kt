package net.mullvad.mullvadvpn.lib.common.util

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import android.util.Log
import kotlin.coroutines.EmptyCoroutineContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.model.ServiceResult

fun <T> SendChannel<T>.safeOffer(element: T): Boolean {
    return runCatching { trySend(element).isSuccess }.getOrDefault(false)
}

fun Context.bindServiceFlow(intent: Intent, flags: Int = 0): Flow<ServiceResult> = callbackFlow {
    val connectionCallback =
        object : ServiceConnection {
            override fun onServiceConnected(className: ComponentName, binder: IBinder) {
                safeOffer(ServiceResult(binder))
            }

            override fun onServiceDisconnected(className: ComponentName) {
                safeOffer(ServiceResult.NOT_CONNECTED)
                bindService(intent, this, flags)
            }
        }

    bindService(intent, connectionCallback, flags)

    awaitClose {
        safeOffer(ServiceResult.NOT_CONNECTED)

        Dispatchers.Default.dispatch(EmptyCoroutineContext) {
            try {
                unbindService(connectionCallback)
            } catch (e: IllegalArgumentException) {
                Log.e("mullvad", "Cannot unbind as no binding exists.")
            }
        }
    }
}
