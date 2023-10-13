package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotification

class NewDeviceNotificationUseCase(private val deviceRepository: DeviceRepository) {
    private val _mutableShowNewDeviceNotification = MutableStateFlow(false)

    fun notifications() =
        combine(
                deviceRepository.deviceState.map { it.deviceName() }.distinctUntilChanged(),
                _mutableShowNewDeviceNotification
            ) { deviceName, newDeviceCreated ->
                if (newDeviceCreated && deviceName != null) {
                    InAppNotification.NewDevice(deviceName)
                } else null
            }
            .map(::listOfNotNull)
            .distinctUntilChanged()

    fun newDeviceCreated() {
        _mutableShowNewDeviceNotification.value = true
    }

    fun clearNewDeviceCreatedNotification() {
        _mutableShowNewDeviceNotification.value = false
    }
}
