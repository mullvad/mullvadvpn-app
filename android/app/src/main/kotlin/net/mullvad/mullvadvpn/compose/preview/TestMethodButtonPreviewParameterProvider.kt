package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.usecase.TestApiAccessMethodState

class TestMethodButtonPreviewParameterProvider :
    PreviewParameterProvider<TestApiAccessMethodState?> {
    override val values: Sequence<TestApiAccessMethodState?> =
        sequenceOf(
            null,
            TestApiAccessMethodState.Testing,
            TestApiAccessMethodState.Successful,
            TestApiAccessMethodState.Failure
        )
}
