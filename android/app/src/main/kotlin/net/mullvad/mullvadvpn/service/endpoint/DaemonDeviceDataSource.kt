package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.util.JobTracker

class DaemonDeviceDataSource(val endpoint: ServiceEndpoint) {
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
        tracker.newBackgroundJob("propagateDeviceUpdatesJob") {
            daemon.deviceStateUpdates.collect { newState ->
                endpoint.sendEvent(Event.DeviceStateEvent(newState))
            }
        }

        tracker.newBackgroundJob("propagateDeviceListUpdatesJob") {
            daemon.deviceListUpdates.collect { newState ->
                endpoint.sendEvent(Event.DeviceListUpdate(newState))
            }
        }

        endpoint.dispatcher.registerHandler(Request.GetDevice::class) {
            tracker.newBackgroundJob("getDeviceJob") { daemon.getAndEmitDeviceState() }
        }

        endpoint.dispatcher.registerHandler(Request.RefreshDeviceState::class) {
            tracker.newBackgroundJob("refreshDeviceJob") { daemon.refreshDevice() }
        }

        endpoint.dispatcher.registerHandler(Request.RemoveDevice::class) { request ->
            tracker.newBackgroundJob("removeDeviceJob") {
                daemon.removeDevice(request.accountToken, request.deviceId).also { result ->
                    endpoint.sendEvent(Event.DeviceRemovalEvent(request.deviceId, result))
                }
            }
        }

        endpoint.dispatcher.registerHandler(Request.GetDeviceList::class) { request ->
            tracker.newBackgroundJob("getDeviceListJob") {
                daemon.getAndEmitDeviceList(request.accountToken)
            }
        }
    }

    fun onDestroy() {
        tracker.cancelAllJobs()
        endpoint.intermittentDaemon.unregisterListener(this)
    }
}
