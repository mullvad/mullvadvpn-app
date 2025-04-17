package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewCreateAccountConfirmationDialog() {
    AppTheme { CreateAccountConfirmation(EmptyResultBackNavigator()) }
}

@Composable
@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
fun CreateAccountConfirmation(navigator: ResultBackNavigator<Confirmed>) {
    InfoConfirmationDialog(
        navigator = navigator,
        titleType = InfoConfirmationDialogTitleType.IconOnly,
        confirmButtonTitle = stringResource(R.string.create_new_account),
        cancelButtonTitle = stringResource(R.string.cancel),
    ) {
        Text(
            text = stringResource(id = R.string.create_new_account_warning_paragraph1),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(Dimens.verticalSpace))

        Text(
            text = stringResource(id = R.string.create_new_account_warning_paragraph2),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
