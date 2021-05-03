package net.mullvad.mullvadvpn.ipc

import android.content.Context
import android.content.Intent
import android.os.IBinder
import android.os.Looper
import android.os.Messenger
import kotlin.reflect.KClass
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.mullvadvpn.util.DispatchingFlow
import net.mullvad.mullvadvpn.util.bindServiceFlow
import net.mullvad.mullvadvpn.util.dispatchTo

@InternalCoroutinesApi
class ServiceConnection(context: Context, scope: CoroutineScope) {
    private val connection = MutableStateFlow<Messenger?>(null)
    private val handler = HandlerFlow(Looper.getMainLooper(), Event::fromMessage)
    private val listener = Messenger(handler)

    private lateinit var requestSinks: StateFlow<Messenger?>

    init {
        val dispatcher = handler
            .filterNotNull()
            .dispatchTo {
                requestSinks = subscribeToState(Event.ListenerReady::class, scope) { connection }
            }

        scope.launch { connect(context) }
        scope.launch { dispatcher.collect() }
        scope.launch { requestSinks.collect { connection.value = it } }
    }

    private suspend fun connect(context: Context) {
        val intent = Intent(context, MullvadVpnService::class.java)

        context.bindServiceFlow(intent).collect { binder ->
            connection.value = null
            binder?.let(::registerListener)
        }
    }

    private fun registerListener(binder: IBinder) {
        val request = Request.RegisterListener(listener)
        val messenger = Messenger(binder)

        messenger.send(request.message)
    }

    private fun <V : Any, D> DispatchingFlow<in V>.subscribeToState(
        event: KClass<V>,
        scope: CoroutineScope,
        dataExtractor: suspend V.() -> D
    ) = subscribe(event).map(dataExtractor).stateIn(scope, SharingStarted.Lazily, null)
}
