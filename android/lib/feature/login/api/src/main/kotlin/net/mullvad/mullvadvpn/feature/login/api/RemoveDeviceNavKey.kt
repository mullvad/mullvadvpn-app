package net.mullvad.mullvadvpn.feature.login.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavigationResult
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId

@Serializable data class RemoveDeviceNavKey(val device: Device) : NavKey

@Serializable
data class RemoveDeviceConfirmationDialogResult(val device: DeviceId) : NavigationResult


