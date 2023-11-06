package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.EmptyDestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewCustomDnsInfoDialog() {
    CustomDnsInfoDialog(EmptyDestinationsNavigator)
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun CustomDnsInfoDialog(navigator: DestinationsNavigator) {
    InfoDialog(
        message = stringResource(id = R.string.settings_changes_effect_warning_content_blocker),
        onDismiss = navigator::navigateUp
    )
}
