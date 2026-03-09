package net.mullvad.mullvadvpn.feature.login.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId

@Parcelize data class RemoveDeviceNavKey(val device: Device) : NavKey2

@Parcelize data class RemoveDeviceConfirmationDialogResult(val device: DeviceId) : NavResult
