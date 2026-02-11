package net.mullvad.mullvadvpn.compose.dialog.info

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.common.compose.clickableAnnotatedString
import net.mullvad.mullvadvpn.common.compose.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.compose.screen.MainGraph
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewAndroid16UpgradeWarningInfoDialog() {
    AppTheme { Android16UpgradeWarningInfoDialog(onDismiss = {}, onClickEmail = {}) }
}

@Destination<MainGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun Android16UpgradeWarningInfo(navigator: DestinationsNavigator) {
    val copyToClipboard = createCopyToClipboardHandle(isSensitive = false)
    Android16UpgradeWarningInfoDialog(
        onDismiss = navigator::navigateUp,
        onClickEmail = { email -> copyToClipboard(email, null) },
    )
}

@Composable
fun Android16UpgradeWarningInfoDialog(onDismiss: () -> Unit, onClickEmail: (String) -> Unit) {
    InfoDialog(
        title = stringResource(id = R.string.android_16_upgrade_warning_title),
        message = stringResource(id = R.string.android_16_upgrade_warning_dialog_first_message),
        additionalInfo =
            clickableAnnotatedString(
                text = stringResource(R.string.android_16_upgrade_warning_dialog_second_message),
                linkStyle =
                    SpanStyle(
                        color = MaterialTheme.colorScheme.onSurface,
                        textDecoration = TextDecoration.Underline,
                    ),
                argument = stringResource(R.string.support_email),
                onClick = onClickEmail,
            ),
        showIcon = false,
        onDismiss = onDismiss,
    )
}
