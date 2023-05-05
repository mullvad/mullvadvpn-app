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

fun Animation.transitionFinished(): Flow<Unit> =
    callbackFlow<Unit> {
            val transitionAnimationListener =
                object : Animation.AnimationListener {
                    override fun onAnimationStart(animation: Animation?) {}
                    override fun onAnimationEnd(animation: Animation?) {
                        safeOffer(Unit)
                    }

                    override fun onAnimationRepeat(animation: Animation?) {}
                }
            setAnimationListener(transitionAnimationListener)
            awaitClose {
                Dispatchers.Main.dispatch(EmptyCoroutineContext) { setAnimationListener(null) }
            }
        }
        .take(1)

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

fun <T> callbackFlowFromNotifier(notifier: EventNotifier<T>) =
    callbackFlow<T> {
        val handler: (T) -> Unit = { value -> trySend(value) }
        notifier.subscribe(this, handler)
        awaitClose { notifier.unsubscribe(this) }
    }

inline fun <T1, T2, T3, T4, T5, T6, R> combine(
    flow: Flow<T1>,
    flow2: Flow<T2>,
    flow3: Flow<T3>,
    flow4: Flow<T4>,
    flow5: Flow<T5>,
    flow6: Flow<T6>,
    crossinline transform: suspend (T1, T2, T3, T4, T5, T6) -> R
): Flow<R> {
    return kotlinx.coroutines.flow.combine(flow, flow2, flow3, flow4, flow5, flow6) { args: Array<*>
        ->
        @Suppress("UNCHECKED_CAST")
        transform(
            args[0] as T1,
            args[1] as T2,
            args[2] as T3,
            args[3] as T4,
            args[4] as T5,
            args[5] as T6,
        )
    }
}
