package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.util.JobTracker

class DaemonDeviceDataSource(
    val endpoint: ServiceEndpoint
) {
    private val tracker = JobTracker()

    init {
        endpoint.intermittentDaemon.registerListener(this) { daemon ->
            if (daemon != null) {
                launchDeviceEndpointJobs(daemon)
            } else {
                tracker.cancelAllJobs()
            }
        }
    }

    private fun launchDeviceEndpointJobs(daemon: MullvadDaemon) {
        tracker.newBackgroundJob("propagateDeviceUpdates") {
            daemon.deviceStateUpdates.collect { newState ->
                endpoint.sendEvent(Event.DeviceStateEvent(newState))
            }
        }

        endpoint.dispatcher.registerHandler(Request.RefreshDeviceState::class) {
            tracker.newBackgroundJob("refreshDeviceJob") {
                daemon.refreshDevice()
            }
        }
    }

    fun onDestroy() {
        tracker.cancelAllJobs()
        endpoint.intermittentDaemon.unregisterListener(this)
    }
}
