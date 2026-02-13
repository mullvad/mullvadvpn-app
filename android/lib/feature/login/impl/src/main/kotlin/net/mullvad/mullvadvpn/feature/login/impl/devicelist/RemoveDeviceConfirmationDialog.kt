package net.mullvad.mullvadvpn.feature.login.impl.devicelist

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.core.text.HtmlCompat
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.feature.managedevices.impl.confirmation.ManageDeviceRemoveConfirmationPreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.ui.component.dialog.NegativeConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.component.toAnnotatedString
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog(
    @PreviewParameter(ManageDeviceRemoveConfirmationPreviewParameterProvider::class) device: Device
) {
    AppTheme { RemoveDeviceConfirmation(EmptyResultBackNavigator(), device = device) }
}

@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun RemoveDeviceConfirmation(navigator: ResultBackNavigator<DeviceId>, device: Device) {
    val htmlFormattedString =
        stringResource(id = R.string.max_devices_confirm_removal_description, device.displayName())
    val message =
        HtmlCompat.fromHtml(htmlFormattedString, HtmlCompat.FROM_HTML_MODE_COMPACT)
            .toAnnotatedString(
                boldSpanStyle =
                    SpanStyle(
                        color = MaterialTheme.colorScheme.onSurface,
                        fontWeight = FontWeight.Bold,
                    )
            )
    NegativeConfirmationDialog(
        message = message,
        messageStyle = MaterialTheme.typography.bodyMedium,
        messageColor = MaterialTheme.colorScheme.onSurfaceVariant,
        confirmationText = stringResource(id = R.string.confirm_removal),
        cancelText = stringResource(id = R.string.back),
        onBack = dropUnlessResumed { navigator.navigateBack() },
        onConfirm = dropUnlessResumed { navigator.navigateBack(result = device.id) },
    )
}
