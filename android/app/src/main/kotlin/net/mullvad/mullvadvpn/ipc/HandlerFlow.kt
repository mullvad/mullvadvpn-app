package net.mullvad.mullvadvpn.ipc

import android.os.Handler
import android.os.Looper
import android.os.Message
import android.util.Log
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedSendChannelException
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.FlowCollector
import kotlinx.coroutines.flow.consumeAsFlow
import kotlinx.coroutines.flow.onCompletion

class HandlerFlow<T>(
    looper: Looper,
    private val extractor: (Message) -> T
) : Handler(looper), Flow<T> {
    private val channel = Channel<T>(Channel.UNLIMITED)
    private val flow = channel.consumeAsFlow().onCompletion {
        removeCallbacksAndMessages(null)
    }

    @InternalCoroutinesApi
    override suspend fun collect(collector: FlowCollector<T>) = flow.collect(collector)

    override fun handleMessage(message: Message) {
        val extractedData = extractor(message)

        try {
            channel.trySendBlocking(extractedData)
        } catch (exception: Exception) {
            when (exception) {
                is ClosedSendChannelException, is CancellationException -> {
                    Log.w("mullvad", "Received a message after HandlerFlow was closed", exception)
                    removeCallbacksAndMessages(null)
                }
                else -> throw exception
            }
        }
    }
}
