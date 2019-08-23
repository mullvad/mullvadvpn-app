package net.mullvad.mullvadvpn.dataproxy

import android.net.VpnService

import kotlinx.coroutines.launch
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import net.mullvad.mullvadvpn.MainActivity
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.TunnelState

class ConnectionProxy(val parentActivity: MainActivity) {
    val daemon = parentActivity.daemon

    private var activeAction: Job? = null

    private val attachListenerJob = attachListener()
    private val fetchInitialStateJob = fetchInitialState()

    private var realState: TunnelState? = null
        set(value) {
            field = value
            uiState = value ?: TunnelState.Disconnected()
        }

    val state: TunnelState
        get() {
            return realState ?: TunnelState.Disconnected()
        }

    var uiState: TunnelState = TunnelState.Disconnected()
        private set(value) {
            field = value
            onUiStateChange?.invoke(value)
        }

    var onUiStateChange: ((TunnelState) -> Unit)? = null
        set(value) {
            field = value
            value?.invoke(uiState)
        }

    var vpnPermission = CompletableDeferred<Boolean>()

    fun connect() {
        if (anticipateConnectingState()) {
            cancelActiveAction()

            requestVpnPermission()

            activeAction = GlobalScope.launch(Dispatchers.Default) {
                if (vpnPermission.await()) {
                    daemon.await().connect()
                }
            }
        }
    }

    fun disconnect() {
        if (anticipateDisconnectingState()) {
            cancelActiveAction()
            activeAction = GlobalScope.launch(Dispatchers.Default) {
                daemon.await().disconnect()
            }
        }
    }

    fun cancelActiveAction() {
        activeAction?.cancel()
    }

    fun onDestroy() {
        attachListenerJob.cancel()
        detachListener()
        fetchInitialStateJob.cancel()
        cancelActiveAction()
    }

    private fun anticipateConnectingState(): Boolean {
        synchronized(this) {
            val currentState = uiState

            if (currentState is TunnelState.Connecting || currentState is TunnelState.Connected) {
                return false
            } else {
                uiState = TunnelState.Connecting(null, null)
                return true
            }
        }
    }

    private fun anticipateDisconnectingState(): Boolean {
        synchronized(this) {
            val currentState = uiState

            if (currentState is TunnelState.Disconnected) {
                return false
            } else {
                uiState = TunnelState.Disconnecting(ActionAfterDisconnect.Nothing())
                return true
            }
        }
    }

    private fun requestVpnPermission() {
        val intent = VpnService.prepare(parentActivity)

        vpnPermission = CompletableDeferred()

        if (intent == null) {
            vpnPermission.complete(true)
        } else {
            parentActivity.requestVpnPermission(intent)
        }
    }

    private fun fetchInitialState() = GlobalScope.launch(Dispatchers.Default) {
        val initialState = daemon.await().getState()

        synchronized(this) {
            if (realState == null) {
                realState = initialState
            }
        }
    }

    private fun attachListener() = GlobalScope.launch(Dispatchers.Default) {
        daemon.await().onTunnelStateChange = { newState ->
            synchronized(this) {
                realState = newState
            }
        }
    }

    private fun detachListener() = GlobalScope.launch(Dispatchers.Default) {
        daemon.await().onTunnelStateChange = null
    }
}
