package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.model.DeviceState

class DeviceRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val deviceState = serviceConnectionManager.connectionState
        .flatMapLatest { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                state.container.deviceDataSource.deviceStateUpdates
                    .onStart {
                        state.container.deviceDataSource.refreshDevice()
                    }
            } else {
                flowOf(DeviceState.Unknown)
            }
        }
        .stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            DeviceState.Initial
        )

    fun refreshDeviceState() {
        container()?.deviceDataSource?.refreshDevice()
    }

    private fun container(): ServiceConnectionContainer? {
        return serviceConnectionManager.connectionState.value.readyContainer()
    }
}
