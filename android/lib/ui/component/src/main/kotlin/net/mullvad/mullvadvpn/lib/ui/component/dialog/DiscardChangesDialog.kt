package net.mullvad.mullvadvpn.lib.ui.component.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.EmptyNavigator
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewDiscardChangesDialog() {
    AppTheme { DiscardChanges(EmptyNavigator) }
}

@Composable
fun DiscardChanges(navigator: Navigator) {
    InfoConfirmationDialog(
        onResult = {
            if (it != null) {
                navigator.goBack(result = DiscardChangesConfirmedNavResult)
            } else {
                navigator.goBack()
            }
        },
        titleType =
            InfoConfirmationDialogTitleType.TitleOnly(stringResource(R.string.discard_changes)),
        confirmButtonTitle = stringResource(R.string.discard),
        cancelButtonTitle = stringResource(R.string.cancel),
    )
}

@Parcelize object DiscardChangesNavKey : NavKey2

@Parcelize object DiscardChangesConfirmedNavResult : NavResult

fun EntryProviderScope<NavKey2>.discardChangesEntry(navigator: Navigator) {
    entry<DiscardChangesNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DiscardChanges(navigator = navigator)
    }
}
