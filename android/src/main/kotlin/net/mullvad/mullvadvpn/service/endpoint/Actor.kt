package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking

abstract class Actor<T : Any>(dispatcher: CoroutineDispatcher = Dispatchers.Default) {
    private val commandChannel = GlobalScope.actor<T>(dispatcher, Channel.UNLIMITED) {
        try {
            for (command in channel) {
                onNewCommand(command)
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Registration queue closed; stop registrator
        }
    }

    protected fun sendBlocking(command: T) = commandChannel.sendBlocking(command)
    protected suspend fun send(command: T) = commandChannel.send(command)
    protected fun closeActor() = commandChannel.close()
    protected abstract suspend fun onNewCommand(command: T)
}
