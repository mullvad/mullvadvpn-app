package net.mullvad.mullvadvpn.feaure.vpnsettings.impl.info

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
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Preview
@Composable
private fun PreviewCustomDnsInfoDialog() {
    CustomDnsInfo(EmptyDestinationsNavigator)
}

@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun CustomDnsInfo(navigator: DestinationsNavigator) {
    InfoDialog(
        message = stringResource(id = R.string.settings_changes_effect_warning_content_blocker),
        onDismiss = dropUnlessResumed { navigator.navigateUp() },
    )
}
