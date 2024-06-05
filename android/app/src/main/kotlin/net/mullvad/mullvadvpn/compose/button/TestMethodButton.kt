package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorSmall
import net.mullvad.mullvadvpn.compose.preview.TestMethodButtonPreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Preview
@Composable
fun PreviewTestMethodButton(
    @PreviewParameter(provider = TestMethodButtonPreviewParameterProvider::class)
    state: TestApiAccessMethodState?
) {
    AppTheme { TestMethodButton(testMethodState = state, onTestMethod = {}) }
}

@Composable
fun TestMethodButton(
    modifier: Modifier = Modifier,
    testMethodState: TestApiAccessMethodState?,
    onTestMethod: () -> Unit
) {
    PrimaryButton(
        modifier = modifier,
        leadingIcon = { Icon(testMethodState = testMethodState) },
        onClick = onTestMethod,
        text =
            stringResource(
                id =
                    when (testMethodState) {
                        TestApiAccessMethodState.Result.Successful -> R.string.api_reachable
                        TestApiAccessMethodState.Result.Failure -> R.string.api_unreachable
                        TestApiAccessMethodState.Testing -> R.string.testing
                        null -> R.string.test_method
                    }
            ),
    )
}

@Composable
private fun Icon(testMethodState: TestApiAccessMethodState?) {
    when (testMethodState) {
        TestApiAccessMethodState.Result.Failure ->
            Box(
                modifier =
                    Modifier.size(Dimens.relayCircleSize)
                        .background(color = MaterialTheme.colorScheme.error, shape = CircleShape)
            )
        TestApiAccessMethodState.Result.Successful -> {
            Box(
                modifier =
                    Modifier.size(Dimens.relayCircleSize)
                        .background(color = MaterialTheme.colorScheme.selected, shape = CircleShape)
            )
        }
        TestApiAccessMethodState.Testing -> {
            MullvadCircularProgressIndicatorSmall()
        }
        null -> {
            /*Show nothing*/
        }
    }
}
