package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.EmptyDestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource

@Preview
@Composable
private fun PreviewApiAccessMethodInfoDialog() {
    ApiAccessMethodInfoDialog(EmptyDestinationsNavigator)
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun ApiAccessMethodInfoDialog(navigator: DestinationsNavigator) {
    InfoDialog(
        message = stringResource(id = R.string.api_access_method_info_first_line),
        additionalInfo =
            buildString {
                appendLine(stringResource(id = R.string.api_access_method_info_second_line))
                appendLine(textResource(id = R.string.api_access_method_info_third_line))
                appendLine(textResource(id = R.string.api_access_method_info_fourth_line))
            },
        onDismiss = navigator::navigateUp
    )
}
