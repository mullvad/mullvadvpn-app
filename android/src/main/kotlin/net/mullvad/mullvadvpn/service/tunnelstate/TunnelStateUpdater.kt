package net.mullvad.mullvadvpn.service.tunnelstate

import android.content.Context
import net.mullvad.mullvadvpn.service.ConnectionProxy
import net.mullvad.mullvadvpn.service.ServiceInstance
import net.mullvad.talpid.util.EventNotifier

class TunnelStateUpdater(context: Context, serviceNotifier: EventNotifier<ServiceInstance?>) {
    private val persistence = Persistence(context)

    private var connectionProxy: ConnectionProxy? = null
    private var stateSubscriptionId: Int? = null

    init {
        serviceNotifier.subscribe { serviceInstance ->
            onNewServiceInstance(serviceInstance)
        }
    }

    private fun onNewServiceInstance(serviceInstance: ServiceInstance?) {
        stateSubscriptionId?.let { id -> connectionProxy?.onStateChange?.unsubscribe(id) }

        connectionProxy = serviceInstance?.connectionProxy?.apply {
            stateSubscriptionId = onStateChange.subscribe { newState ->
                persistence.state = newState
            }
        }
    }
}
