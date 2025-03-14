package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.repository.NewDeviceRepository

class NewDeviceNotificationUseCase(
    private val newDeviceRepository: NewDeviceRepository,
    private val deviceRepository: DeviceRepository,
) {
    operator fun invoke() =
        combine(
                deviceRepository.deviceState.map { it?.displayName() },
                newDeviceRepository.isNewDevice,
            ) { deviceName, newDeviceCreated ->
                if (newDeviceCreated && deviceName != null) {
                    InAppNotification.NewDevice(deviceName)
                } else null
            }
            .map(::listOfNotNull)
            .distinctUntilChanged()
}
