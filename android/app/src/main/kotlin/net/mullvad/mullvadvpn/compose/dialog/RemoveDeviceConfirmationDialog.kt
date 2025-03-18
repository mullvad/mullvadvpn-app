package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.core.text.HtmlCompat
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.preview.DevicePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.component.toAnnotatedString

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog(
    @PreviewParameter(DevicePreviewParameterProvider::class) device: Device
) {
    AppTheme { RemoveDeviceConfirmation(EmptyResultBackNavigator(), device = device) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun RemoveDeviceConfirmation(navigator: ResultBackNavigator<DeviceId>, device: Device) {
    val htmlFormattedString =
        textResource(id = R.string.max_devices_confirm_removal_description, device.displayName())
    val message =
        HtmlCompat.fromHtml(htmlFormattedString, HtmlCompat.FROM_HTML_MODE_COMPACT)
            .toAnnotatedString(boldFontWeight = FontWeight.Bold)
    NegativeConfirmationDialog(
        message = message,
        messageStyle = MaterialTheme.typography.labelLarge,
        confirmationText = stringResource(id = R.string.confirm_removal),
        cancelText = stringResource(id = R.string.back),
        onBack = dropUnlessResumed { navigator.navigateBack() },
        onConfirm = dropUnlessResumed { navigator.navigateBack(result = device.id) },
    )
}
