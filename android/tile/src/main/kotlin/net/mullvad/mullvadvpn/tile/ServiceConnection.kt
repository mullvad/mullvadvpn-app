package net.mullvad.mullvadvpn.tile

import android.content.Context
import android.content.Intent
import android.os.IBinder
import android.os.Looper
import android.os.Messenger
import kotlin.reflect.KClass
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.consumeAsFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.common.util.DispatchingFlow
import net.mullvad.mullvadvpn.lib.common.util.bindServiceFlow
import net.mullvad.mullvadvpn.lib.common.util.dispatchTo
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.HandlerFlow
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.ServiceResult
import net.mullvad.mullvadvpn.model.TunnelState

@FlowPreview
class ServiceConnection(context: Context, scope: CoroutineScope) {
    private val activeListeners = MutableStateFlow<Pair<Messenger, Int>?>(null)
    private val handler = HandlerFlow(Looper.getMainLooper(), Event.Companion::fromMessage)
    private val listener = Messenger(handler)
    private val listenerId = MutableStateFlow<Int?>(null)

    private lateinit var listenerRegistrations: StateFlow<Pair<Messenger, Int>?>

    lateinit var tunnelState: Flow<Pair<TunnelState, ServiceResult.ConnectionState>>
        private set

    private val serviceConnectionStateChannel =
        Channel<ServiceResult.ConnectionState>(Channel.RENDEZVOUS)

    init {
        val dispatcher =
            handler.filterNotNull().dispatchTo {
                listenerRegistrations =
                    subscribeToState(Event.ListenerReady::class, scope) {
                        Pair(connection, listenerId)
                    }

                val tunnelStateEvents =
                    subscribeToState(
                        Event.TunnelStateChange::class,
                        scope,
                        TunnelState.Disconnected
                    ) {
                        tunnelState
                    }

                tunnelState =
                    tunnelStateEvents.combine(serviceConnectionStateChannel.consumeAsFlow()) {
                        tunnelState,
                        serviceConnectionState ->
                        tunnelState to serviceConnectionState
                    }
            }

        scope.launch { connect(context) }
        scope.launch { dispatcher.collect() }
        scope.launch { unregisterOldListeners() }
        scope.launch { listenerRegistrations.collect { activeListeners.value = it } }
    }

    private suspend fun connect(context: Context) {
        val intent = Intent().apply { setClassName(context.packageName, VPN_SERVICE_CLASS) }

        context
            .bindServiceFlow(intent)
            .onStart { emit(ServiceResult.NOT_CONNECTED) }
            .onEach { result -> serviceConnectionStateChannel.send(result.connectionState) }
            .collect { result ->
                activeListeners.value = null
                result.binder?.let(::registerListener)
            }
    }

    private fun registerListener(binder: IBinder) {
        val request = Request.RegisterListener(listener)
        val messenger = Messenger(binder)

        messenger.send(request.message)
    }

    private suspend fun unregisterOldListeners() {
        var oldListener: Pair<Messenger, Int>? = null

        activeListeners
            .onCompletion { oldListener?.let(::unregisterListener) }
            .collect { newListener ->
                oldListener?.let(::unregisterListener)
                oldListener = newListener
            }
    }

    private fun unregisterListener(registration: Pair<Messenger, Int>) {
        val (messenger, listenerId) = registration
        val request = Request.UnregisterListener(listenerId)

        messenger.send(request.message)
    }

    private fun <V : Any, D> DispatchingFlow<in V>.subscribeToState(
        event: KClass<V>,
        scope: CoroutineScope,
        dataExtractor: suspend V.() -> D
    ) = subscribe(event).map(dataExtractor).stateIn(scope, SharingStarted.Lazily, null)

    private fun <V : Any, D> DispatchingFlow<in V>.subscribeToState(
        event: KClass<V>,
        scope: CoroutineScope,
        initialValue: D,
        dataExtractor: suspend V.() -> D
    ) = subscribe(event).map(dataExtractor).stateIn(scope, SharingStarted.Lazily, initialValue)
}
