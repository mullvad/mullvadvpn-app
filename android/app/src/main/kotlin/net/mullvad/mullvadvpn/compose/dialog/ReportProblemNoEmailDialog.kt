package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewReportProblemNoEmailDialog() {
    AppTheme { ReportProblemNoEmail(EmptyResultBackNavigator()) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ReportProblemNoEmail(resultBackNavigator: ResultBackNavigator<Boolean>) {
    NegativeConfirmationDialog(
        message = stringResource(id = R.string.confirm_no_email),
        errorMessage = null,
        confirmationText = stringResource(id = R.string.send_anyway),
        cancelText = stringResource(id = R.string.back),
        messageStyle = MaterialTheme.typography.bodySmall,
        onBack = dropUnlessResumed { resultBackNavigator.navigateBack() },
        onConfirm = dropUnlessResumed { resultBackNavigator.navigateBack(result = true) },
    )
}
