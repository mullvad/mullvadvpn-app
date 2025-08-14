package net.mullvad.mullvadvpn.compose.dialog.info

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.fromHtml
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.window.DialogProperties
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.common.util.openAppDetailsSettings
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar

@Preview
@Composable
private fun PreviewApiAccessMethodInfoDialog() {
    AppTheme { ConnectOnStartupInfoDialog({}) }
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
fun ConnectOnStartupInfoDialog(onDismiss: () -> Unit, openAppDetails: () -> Unit = {}) {
    AlertDialog(
        onDismissRequest = { onDismiss() },
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                imageVector = Icons.Default.Info,
                contentDescription = "",
                tint = MaterialTheme.colorScheme.onSurface,
            )
        },
        text = {
            val scrollState = rememberScrollState()
            Column(
                Modifier.drawVerticalScrollbar(
                        scrollState,
                        MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar),
                    )
                    .verticalScroll(scrollState),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    text =
                        buildAnnotatedString {
                            append(stringResource(R.string.connect_on_start_info_first))
                            appendLine(
                                AnnotatedString.fromHtml(
                                    stringResource(R.string.connect_on_start_info_second)
                                )
                            )
                        },
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.labelLarge,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        },
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
        properties = DialogProperties(dismissOnClickOutside = true, dismissOnBackPress = true),
        containerColor = MaterialTheme.colorScheme.surface,
    )
}
