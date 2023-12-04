package net.mullvad.mullvadvpn.compose.dialog

import android.os.Parcelable
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.EmptyDestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.util.asString

@Preview
@Composable
private fun PreviewWireguardPortInfoDialog() {
    AppTheme {
        WireguardPortInfoDialog(
            EmptyDestinationsNavigator,
            argument = WireguardPortInfoDialogArgument(listOf(PortRange(1, 2)))
        )
    }
}

@Parcelize data class WireguardPortInfoDialogArgument(val portRanges: List<PortRange>) : Parcelable

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun WireguardPortInfoDialog(
    navigator: DestinationsNavigator,
    argument: WireguardPortInfoDialogArgument
) {
    InfoDialog(
        message = stringResource(id = R.string.wireguard_port_info_description),
        additionalInfo =
            stringResource(
                id = R.string.wireguard_port_info_port_range,
                argument.portRanges.asString()
            ),
        onDismiss = navigator::navigateUp
    )
}
