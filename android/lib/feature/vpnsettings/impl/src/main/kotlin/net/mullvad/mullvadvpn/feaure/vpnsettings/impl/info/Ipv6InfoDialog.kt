package net.mullvad.mullvadvpn.feaure.vpnsettings.impl.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun Ipv6Info(navigator: DestinationsNavigator) {
    InfoDialog(
        message = stringResource(R.string.ipv6_info),
        onDismiss = dropUnlessResumed { navigator.navigateUp() },
    )
}
