package net.mullvad.mullvadvpn.feature.daita.impl

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.EmptyDestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewDaitaDirectOnlyInfoDialog() {
    AppTheme { DaitaDirectOnlyInfo(EmptyDestinationsNavigator) }
}

@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun DaitaDirectOnlyInfo(navigator: DestinationsNavigator) {
    InfoDialog(
        message =
            stringResource(
                id = R.string.daita_info,
                stringResource(id = R.string.direct_only),
                stringResource(id = R.string.daita),
            ),
        onDismiss = dropUnlessResumed { navigator.navigateUp() },
    )
}
