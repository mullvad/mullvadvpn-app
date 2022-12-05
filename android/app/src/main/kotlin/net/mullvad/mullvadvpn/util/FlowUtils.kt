package net.mullvad.mullvadvpn.util

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import android.util.Log
import android.view.animation.Animation
import kotlin.coroutines.EmptyCoroutineContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.take
import net.mullvad.mullvadvpn.model.ServiceResult
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.talpid.util.EventNotifier

fun <T> SendChannel<T>.safeOffer(element: T): Boolean {
    return runCatching { trySend(element).isSuccess }.getOrDefault(false)
}

fun Animation.transitionFinished(): Flow<Unit> = callbackFlow<Unit> {
    val transitionAnimationListener = object : Animation.AnimationListener {
        override fun onAnimationStart(animation: Animation?) {}
        override fun onAnimationEnd(animation: Animation?) {
            safeOffer(Unit)
        }

        override fun onAnimationRepeat(animation: Animation?) {}
    }
    setAnimationListener(transitionAnimationListener)
    awaitClose {
        Dispatchers.Main.dispatch(EmptyCoroutineContext) {
            setAnimationListener(null)
        }
    }
}.take(1)

fun Context.bindServiceFlow(intent: Intent, flags: Int = 0): Flow<ServiceResult> = callbackFlow {
    val connectionCallback = object : ServiceConnection {
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

fun <R> Flow<ServiceConnectionState>.flatMapReadyConnectionOrDefault(
    default: Flow<R>,
    transform: (value: ServiceConnectionState.ConnectedReady) -> Flow<R>
): Flow<R> {
    return flatMapLatest { state ->
        if (state is ServiceConnectionState.ConnectedReady) {
            transform.invoke(state)
        } else {
            default
        }
    }
}

fun <T> callbackFlowFromNotifier(notifier: EventNotifier<T>) = callbackFlow<T> {
    val handler: (T) -> Unit = { value -> trySend(value) }
    notifier.subscribe(this, handler)
    awaitClose { notifier.unsubscribe(this) }
}
