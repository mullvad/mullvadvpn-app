package net.mullvad.mullvadvpn.repository

import arrow.core.Either
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AccountToken
import net.mullvad.mullvadvpn.lib.model.DeleteDeviceError
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError

class DeviceRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val deviceState: StateFlow<Device?> =
        managementService.deviceState
            .map {
                when (it) {
                    is DeviceState.LoggedIn -> it.device
                    DeviceState.LoggedOut -> null
                    DeviceState.Revoked -> null
                }
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    suspend fun removeDevice(
        accountToken: AccountToken,
        deviceId: DeviceId
    ): Either<DeleteDeviceError, Unit> = managementService.removeDevice(accountToken, deviceId)

    suspend fun deviceList(
        accountToken: AccountToken
    ): Either<net.mullvad.mullvadvpn.lib.model.GetDeviceListError, List<Device>> =
        managementService.getDeviceList(accountToken)
}
