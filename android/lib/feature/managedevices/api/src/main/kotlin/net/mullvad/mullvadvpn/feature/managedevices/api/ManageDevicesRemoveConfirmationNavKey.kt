package net.mullvad.mullvadvpn.feature.managedevices.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId

@Parcelize data class ManageDevicesRemoveConfirmationNavKey(val device: Device) : NavKey2

@Parcelize data class ManageDevicesRemoveConfirmationNavResult(val deviceId: DeviceId) : NavResult
