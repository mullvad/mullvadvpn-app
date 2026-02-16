package net.mullvad.mullvadvpn.feature.apiaccess.impl.component

import androidx.compose.ui.tooling.preview.PreviewParameterProvider

class TestMethodButtonPreviewParameterProvider : PreviewParameterProvider<Boolean> {
    override val values: Sequence<Boolean> = sequenceOf(false, true)
}
