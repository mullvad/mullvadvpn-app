package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.service.persistence.SplitTunnelingPersistence
import net.mullvad.talpid.util.EventNotifier

class SplitTunneling(persistence: SplitTunnelingPersistence, endpoint: ServiceEndpoint) {
    private val excludedApps = persistence.excludedApps.toMutableSet()

    private var enabled by
        observable(persistence.enabled) { _, wasEnabled, isEnabled ->
            if (wasEnabled != isEnabled) {
                persistence.enabled = isEnabled
                update()
            }
        }

    val onChange =
        EventNotifier(
            if (enabled) {
                excludedApps.toList()
            } else {
                null
            }
        )

    init {
        onChange.subscribe(this) { excludedApps ->
            endpoint.sendEvent(Event.SplitTunnelingUpdate(excludedApps))
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.IncludeApp::class) { request ->
                excludedApps.remove(request.packageName)
                update()
            }

            registerHandler(Request.ExcludeApp::class) { request ->
                excludedApps.add(request.packageName)
                update()
            }

            registerHandler(Request.SetEnableSplitTunneling::class) { request ->
                enabled = request.enable
            }

            registerHandler(Request.PersistExcludedApps::class) { _ ->
                persistence.excludedApps = excludedApps
            }
        }
    }

    fun onDestroy() {
        onChange.unsubscribeAll()
    }

    private fun update() {
        if (enabled) {
            onChange.notify(excludedApps.toList())
        } else {
            onChange.notify(null)
        }
    }
}
