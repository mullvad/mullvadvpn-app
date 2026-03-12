package net.mullvad.mullvadvpn.feature.managedevices.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId

@Parcelize
data class ManageDevicesRemoveConfirmationNavKey(val device: Device) : NavKey2

@Parcelize
data class ManageDevicesRemoveConfirmationNavResult(val deviceId: DeviceId) : NavResult

