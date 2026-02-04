package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.MainGraph
import net.mullvad.mullvadvpn.lib.ui.component.dialog.Confirmed
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialogTitleType
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewDiscardChangesDialog() {
    AppTheme { DiscardChanges(EmptyResultBackNavigator()) }
}

@Destination<MainGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun DiscardChanges(resultBackNavigator: ResultBackNavigator<Confirmed>) {
    InfoConfirmationDialog(
        onResult = {
            if (it != null) {
                resultBackNavigator.navigateBack(it)
            } else {
                resultBackNavigator.navigateBack()
            }
        },
        titleType =
            InfoConfirmationDialogTitleType.TitleOnly(stringResource(R.string.discard_changes)),
        confirmButtonTitle = stringResource(R.string.discard),
        cancelButtonTitle = stringResource(R.string.cancel),
    )
}
