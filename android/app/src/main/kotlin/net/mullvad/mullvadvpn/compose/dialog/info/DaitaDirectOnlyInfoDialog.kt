package net.mullvad.mullvadvpn.compose.dialog.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.EmptyDestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewDaitaDirectOnlyInfoDialog() {
    AppTheme { DaitaDirectOnlyInfo(EmptyDestinationsNavigator) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
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
