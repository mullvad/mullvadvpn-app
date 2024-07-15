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
private fun PreviewObfuscationInfoDialog() {
    ObfuscationInfo(EmptyDestinationsNavigator)
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ObfuscationInfo(navigator: DestinationsNavigator) {
    InfoDialog(
        message = stringResource(id = R.string.obfuscation_info),
        onDismiss = dropUnlessResumed { navigator.navigateUp() }
    )
}
