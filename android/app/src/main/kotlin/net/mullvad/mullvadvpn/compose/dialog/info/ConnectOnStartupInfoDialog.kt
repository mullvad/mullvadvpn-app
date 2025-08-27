package net.mullvad.mullvadvpn.compose.dialog.info

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.lib.common.util.openAppDetailsSettings
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewApiAccessMethodInfoDialog() {
    AppTheme { ConnectOnStartupInfoDialog(onDismiss = {}, openAppDetails = {}) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ConnectOnStartupInfo(navigator: DestinationsNavigator) {
    val context = LocalContext.current
    ConnectOnStartupInfoDialog(
        onDismiss = navigator::navigateUp,
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
                        imageVector = Icons.AutoMirrored.Filled.OpenInNew,
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
