package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.discardchanges

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.lib.ui.component.dialog.Confirmed
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialogTitleType
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewApiAccessDiscardChangesDialog() {
    AppTheme { DiscardApiAccessChanges(EmptyResultBackNavigator()) }
}

@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun DiscardApiAccessChanges(resultBackNavigator: ResultBackNavigator<Confirmed>) {
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
