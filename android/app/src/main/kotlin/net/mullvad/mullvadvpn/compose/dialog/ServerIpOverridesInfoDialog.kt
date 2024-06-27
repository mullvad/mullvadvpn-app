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

@Preview
@Composable
private fun PreviewServerIpOverridesInfoDialog() {
    ServerIpOverridesInfoDialog(EmptyDestinationsNavigator)
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ServerIpOverridesInfoDialog(navigator: DestinationsNavigator) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.server_ip_overrides_info_first_paragraph))
                appendLine()
                appendLine(stringResource(id = R.string.server_ip_overrides_info_second_paragraph))
                appendLine()
                append(stringResource(id = R.string.server_ip_overrides_info_third_paragraph))
            },
        onDismiss = dropUnlessResumed { navigator.navigateUp() }
    )
}
