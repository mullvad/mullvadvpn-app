package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.dialog.info.Confirmed
import net.mullvad.mullvadvpn.compose.dialog.info.InfoConfirmationDialog
import net.mullvad.mullvadvpn.compose.dialog.info.InfoConfirmationDialogTitleType
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewDaitaDirectOnlyConfirmationDialog() {
    AppTheme { DaitaDirectOnlyConfirmation(EmptyResultBackNavigator()) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun DaitaDirectOnlyConfirmation(navigator: ResultBackNavigator<Confirmed>) {
    InfoConfirmationDialog(
        navigator = navigator,
        titleType = InfoConfirmationDialogTitleType.IconOnly,
        confirmButtonTitle =
            stringResource(R.string.enable_direct_only, stringResource(R.string.direct_only)),
        cancelButtonTitle = stringResource(R.string.cancel),
    ) {
        Text(
            text =
                stringResource(
                    id = R.string.direct_only_description,
                    stringResource(id = R.string.daita),
                ),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
