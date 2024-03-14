package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.stringResource
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun DiscardChangesDialog(resultBackNavigator: ResultBackNavigator<Boolean>) {
    AlertDialog(
        onDismissRequest = resultBackNavigator::navigateBack,
        title = { Text(text = stringResource(id = R.string.discard_changes)) },
        dismissButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = resultBackNavigator::navigateBack,
                text = stringResource(id = R.string.cancel)
            )
        },
        confirmButton = {
            PrimaryButton(
                onClick = { resultBackNavigator.navigateBack(result = true) },
                text = stringResource(id = R.string.discard)
            )
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}
