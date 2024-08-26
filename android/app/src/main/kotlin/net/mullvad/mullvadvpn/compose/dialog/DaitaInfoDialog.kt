package net.mullvad.mullvadvpn.compose.dialog

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
private fun PreviewDaitaInfoDialog() {
    AppTheme { DaitaInfo(EmptyDestinationsNavigator) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun DaitaInfo(navigator: DestinationsNavigator) {
    InfoDialog(
        message =
            stringResource(
                id = R.string.daita_info,
                stringResource(id = R.string.daita),
                stringResource(id = R.string.daita_full),
            ),
        additionalInfo =
            stringResource(id = R.string.daita_warning, stringResource(id = R.string.daita)),
        onDismiss = dropUnlessResumed { navigator.navigateUp() },
    )
}
