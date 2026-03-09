package net.mullvad.mullvadvpn.feature.vpnsettings.impl.info

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.util.openAppDetailsSettings
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewApiAccessMethodInfoDialog() {
    AppTheme { ConnectOnStartupInfoDialog(onDismiss = {}, openAppDetails = {}) }
}

@Composable
fun ConnectOnStartupInfo(navigator: Navigator) {
    val context = LocalContext.current
    ConnectOnStartupInfoDialog(
        onDismiss = navigator::goBack,
        openAppDetails = { context.openAppDetailsSettings() },
    )
}

@Composable
fun ConnectOnStartupInfoDialog(onDismiss: () -> Unit, openAppDetails: () -> Unit) {
    InfoDialog(
        onDismiss = { onDismiss() },
        message = stringResource(R.string.connect_on_start_info_first),
        additionalInfo = stringResource(R.string.connect_on_start_info_second),
        confirmButton = {
            PrimaryButton(
                text = stringResource(R.string.open_app_details),
                onClick = openAppDetails,
                trailingIcon = {
                    Icon(
                        imageVector = Icons.AutoMirrored.Rounded.OpenInNew,
                        tint = MaterialTheme.colorScheme.onPrimary,
                        contentDescription = null,
                    )
                },
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(R.string.got_it), onClick = onDismiss)
        },
    )
}
