package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotification

class NewDeviceNotificationUseCase(private val deviceRepository: DeviceRepository) {
    private val _mutableShowNewDeviceNotification = MutableStateFlow(false)

    fun notifications() =
        combine(
                deviceRepository.deviceState
                    .mapNotNull { it?.displayName() }
                    .distinctUntilChanged(),
                _mutableShowNewDeviceNotification
            ) { deviceName, newDeviceCreated ->
                if (newDeviceCreated) {
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
