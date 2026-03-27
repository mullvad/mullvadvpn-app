package net.mullvad.mullvadvpn.feature.daita.impl

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.core.EmptyNavigator
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.daita.api.DaitaDirectOnlyConfirmedNavResult
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialogTitleType
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewDaitaDirectOnlyConfirmationDialog() {
    AppTheme { DaitaDirectOnlyConfirmation(EmptyNavigator) }
}

@Composable
fun DaitaDirectOnlyConfirmation(navigator: Navigator) {
    InfoConfirmationDialog(
        onResult = {
            if (it != null) {
                navigator.goBack(result = DaitaDirectOnlyConfirmedNavResult)
            } else {
                navigator.goBack()
            }
        },
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
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelLarge,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
