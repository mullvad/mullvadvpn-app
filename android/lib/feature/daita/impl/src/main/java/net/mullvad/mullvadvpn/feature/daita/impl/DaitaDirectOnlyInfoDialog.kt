package net.mullvad.mullvadvpn.feature.daita.impl

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog

@Preview
@Composable
private fun PreviewDaitaDirectOnlyInfoDialog() {
    //    AppTheme { DaitaDirectOnlyInfo(EmptyDestinationsNavigator) }
}

@Composable
fun DaitaDirectOnlyInfo(navigator: Navigator) {
    InfoDialog(
        message =
            stringResource(
                id = R.string.daita_info,
                stringResource(id = R.string.direct_only),
                stringResource(id = R.string.daita),
            ),
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}
