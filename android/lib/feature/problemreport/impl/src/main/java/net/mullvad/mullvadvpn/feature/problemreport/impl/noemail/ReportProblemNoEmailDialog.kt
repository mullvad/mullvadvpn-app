package net.mullvad.mullvadvpn.feature.problemreport.impl.noemail

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.EmptyNavigator
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.problemreport.api.ProblemReportNoEmailConfirmedNavResult
import net.mullvad.mullvadvpn.lib.ui.component.dialog.NegativeConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewReportProblemNoEmailDialog() {
    AppTheme { ReportProblemNoEmail(EmptyNavigator) }
}

@Composable
fun ReportProblemNoEmail(navigator: Navigator) {
    NegativeConfirmationDialog(
        message = stringResource(id = R.string.confirm_no_email),
        confirmationText = stringResource(id = R.string.send_anyway),
        cancelText = stringResource(id = R.string.back),
        messageStyle = MaterialTheme.typography.labelLarge,
        messageColor = MaterialTheme.colorScheme.onSurfaceVariant,
        onBack = dropUnlessResumed { navigator.goBack() },
        onConfirm =
            dropUnlessResumed { navigator.goBack(result = ProblemReportNoEmailConfirmedNavResult) },
    )
}
