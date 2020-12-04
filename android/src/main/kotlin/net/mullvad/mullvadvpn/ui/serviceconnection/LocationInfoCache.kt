package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.service.Event

class LocationInfoCache(val eventDispatcher: EventDispatcher) {
    private var location: GeoIpLocation? by observable(null) { _, _, newLocation ->
        onNewLocation?.invoke(newLocation)
    }

    var onNewLocation by observable<((GeoIpLocation?) -> Unit)?>(null) { _, _, callback ->
        callback?.invoke(location)
    }

    init {
        eventDispatcher.registerHandler(Event.Type.NewLocation) { event: Event.NewLocation ->
            location = event.location
        }
    }
}
