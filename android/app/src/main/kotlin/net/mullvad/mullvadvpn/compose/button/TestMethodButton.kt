package net.mullvad.mullvadvpn.compose.button

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.preview.TestMethodButtonPreviewParameterProvider
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewTestMethodButton(
    @PreviewParameter(provider = TestMethodButtonPreviewParameterProvider::class) isTesting: Boolean
) {
    AppTheme { TestMethodButton(isTesting = isTesting, onTestMethod = {}) }
}

@Composable
fun TestMethodButton(modifier: Modifier = Modifier, isTesting: Boolean, onTestMethod: () -> Unit) {
    PrimaryButton(
        modifier = modifier,
        onClick = onTestMethod,
        isEnabled = !isTesting,
        text =
            stringResource(
                id =
                    if (isTesting) {
                        R.string.testing
                    } else {
                        R.string.test_method
                    }
            ),
    )
}
