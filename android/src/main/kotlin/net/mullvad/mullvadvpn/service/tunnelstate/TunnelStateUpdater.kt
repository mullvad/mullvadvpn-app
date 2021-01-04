package net.mullvad.mullvadvpn.service.tunnelstate

import android.content.Context
import net.mullvad.mullvadvpn.service.ServiceInstance
import net.mullvad.mullvadvpn.service.endpoint.ConnectionProxy
import net.mullvad.talpid.util.EventNotifier

class TunnelStateUpdater(context: Context, serviceNotifier: EventNotifier<ServiceInstance?>) {
    private val persistence = Persistence(context)

    private var connectionProxy: ConnectionProxy? = null
    private var stateSubscriptionId: Int? = null

    init {
        serviceNotifier.subscribe(this) { serviceInstance ->
            onNewServiceInstance(serviceInstance)
        }
    }

    private fun onNewServiceInstance(serviceInstance: ServiceInstance?) {
        connectionProxy?.onStateChange?.unsubscribe(this)

        connectionProxy = serviceInstance?.connectionProxy?.apply {
            onStateChange.subscribe(this@TunnelStateUpdater) { newState ->
                persistence.state = newState
            }
        }
    }
}
