package net.mullvad.mullvadvpn.repository

import arrow.core.Either
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.DeleteDeviceError
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceId
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.GetDeviceListError

class DeviceRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val deviceState: StateFlow<DeviceState> =
        managementService.deviceState.stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.Eagerly,
            DeviceState.Initial
        )

    suspend fun removeDevice(
        accountToken: AccountToken,
        deviceId: DeviceId
    ): Either<DeleteDeviceError, Unit> = managementService.removeDevice(accountToken, deviceId)

    suspend fun deviceList(accountToken: AccountToken): Either<GetDeviceListError, List<Device>> =
        managementService.getDeviceList(accountToken)
}
