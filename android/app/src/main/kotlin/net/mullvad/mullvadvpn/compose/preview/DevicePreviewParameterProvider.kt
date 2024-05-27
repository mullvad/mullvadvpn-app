package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Device

class DevicePreviewParameterProvider : PreviewParameterProvider<Device> {
    override val values: Sequence<Device> = sequenceOf(generateDevice())
}
