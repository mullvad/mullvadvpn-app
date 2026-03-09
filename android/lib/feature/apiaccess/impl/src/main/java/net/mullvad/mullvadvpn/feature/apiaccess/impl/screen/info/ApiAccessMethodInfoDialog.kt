package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Preview
@Composable
private fun PreviewApiAccessMethodInfoDialog() {
    //    AppTheme { ApiAccessMethodInfo(EmptyDestinationsNavigator) }
}

@Composable
fun ApiAccessMethodInfo(navigator: Navigator) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.api_access_method_info_first_line))
                appendLine()
                appendLine(stringResource(id = R.string.api_access_method_info_second_line))
                appendLine()
                appendLine(stringResource(id = R.string.api_access_method_info_third_line))
                appendLine()
                appendLine(stringResource(id = R.string.api_access_method_info_fourth_line))
            },
        onDismiss = navigator::goBack,
    )
}
