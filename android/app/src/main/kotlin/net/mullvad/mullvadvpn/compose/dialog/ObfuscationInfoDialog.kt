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
private fun PreviewObfuscationInfoDialog() {
    ObfuscationInfoDialog(EmptyDestinationsNavigator)
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun ObfuscationInfoDialog(navigator: DestinationsNavigator) {
    InfoDialog(
        message = stringResource(id = R.string.obfuscation_info),
        onDismiss = navigator::navigateUp
    )
}
