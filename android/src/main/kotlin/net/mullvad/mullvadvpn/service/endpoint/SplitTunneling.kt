package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event.SplitTunnelingUpdate
import net.mullvad.mullvadvpn.ipc.Request.ExcludeApp
import net.mullvad.mullvadvpn.ipc.Request.IncludeApp
import net.mullvad.mullvadvpn.ipc.Request.PersistExcludedApps
import net.mullvad.mullvadvpn.ipc.Request.SetEnableSplitTunneling
import net.mullvad.mullvadvpn.service.persistence.SplitTunnelingPersistence
import net.mullvad.talpid.util.EventNotifier

class SplitTunneling(persistence: SplitTunnelingPersistence, endpoint: ServiceEndpoint) {
    private val excludedApps = persistence.excludedApps.toMutableSet()

    private var enabled by observable(persistence.enabled) { _, _, isEnabled ->
        persistence.enabled = isEnabled
        update()
    }

    val onChange = EventNotifier<List<String>?>(null)

    init {
        onChange.subscribe(this) { excludedApps ->
            endpoint.sendEvent(SplitTunnelingUpdate(excludedApps))
        }

        endpoint.dispatcher.run {
            registerHandler(IncludeApp::class) { request ->
                excludedApps.remove(request.packageName)
                update()
            }

            registerHandler(ExcludeApp::class) { request ->
                excludedApps.add(request.packageName)
                update()
            }

            registerHandler(SetEnableSplitTunneling::class) { request -> enabled = request.enable }
            registerHandler(PersistExcludedApps::class) { persistence.excludedApps = excludedApps }
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
