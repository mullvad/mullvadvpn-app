package net.mullvad.mullvadvpn.util

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import android.view.animation.Animation
import kotlin.coroutines.EmptyCoroutineContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.take

fun <T> SendChannel<T>.safeOffer(element: T): Boolean {
    return runCatching { offer(element) }.getOrDefault(false)
}

fun Animation.transitionFinished(): Flow<Unit> = callbackFlow<Unit> {
    val transitionAnimationListener = object : Animation.AnimationListener {
        override fun onAnimationStart(animation: Animation?) {}
        override fun onAnimationEnd(animation: Animation?) { safeOffer(Unit) }
        override fun onAnimationRepeat(animation: Animation?) {}
    }
    setAnimationListener(transitionAnimationListener)
    awaitClose {
        Dispatchers.Main.dispatch(EmptyCoroutineContext) {
            setAnimationListener(null)
        }
    }
}.take(1)

fun Context.bindServiceFlow(intent: Intent, flags: Int = 0): Flow<IBinder?> = callbackFlow {
    val connectionCallback = object : ServiceConnection {
        override fun onServiceConnected(className: ComponentName, binder: IBinder) {
            safeOffer(binder)
        }

        override fun onServiceDisconnected(className: ComponentName) {
            safeOffer(null)
        }
    }

    bindService(intent, connectionCallback, flags)

    awaitClose {
        safeOffer(null)

        Dispatchers.Default.dispatch(EmptyCoroutineContext) {
            unbindService(connectionCallback)
        }
    }
}
