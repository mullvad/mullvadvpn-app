package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class NewDeviceRepository {
    private val _mutableShowNewDeviceNotification = MutableStateFlow(false)

    val isNewDevice: StateFlow<Boolean> = _mutableShowNewDeviceNotification

    fun newDeviceCreated() {
        _mutableShowNewDeviceNotification.value = true
    }

    fun clearNewDeviceCreatedNotification() {
        _mutableShowNewDeviceNotification.value = false
    }
}
