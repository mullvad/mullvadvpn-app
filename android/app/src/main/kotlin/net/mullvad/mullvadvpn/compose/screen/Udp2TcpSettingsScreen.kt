package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.UdpOverTcpPortInfoDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.InformationComposeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.Udp2TcpSettingsState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.constant.UDP2TCP_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.tag.UDP_OVER_TCP_PORT_ITEM_AUTOMATIC_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.UDP_OVER_TCP_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.viewmodel.Udp2TcpSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewUdp2TcpSettingsScreen() {
    AppTheme {
        Udp2TcpSettingsScreen(
            state = Udp2TcpSettingsState(port = Constraint.Any),
            onObfuscationPortSelected = {},
            navigateUdp2TcpInfo = {},
            onBackClick = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun Udp2TcpSettings(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<Udp2TcpSettingsViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    Udp2TcpSettingsScreen(
        state = state,
        onObfuscationPortSelected = viewModel::onObfuscationPortSelected,
        navigateUdp2TcpInfo =
            dropUnlessResumed { navigator.navigate(UdpOverTcpPortInfoDestination) },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun Udp2TcpSettingsScreen(
    state: Udp2TcpSettingsState,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
    navigateUdp2TcpInfo: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.upd_over_tcp),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(modifier = modifier, state = lazyListState) {
            itemWithDivider {
                InformationComposeCell(
                    title = stringResource(R.string.port),
                    onInfoClicked = navigateUdp2TcpInfo,
                    onCellClicked = navigateUdp2TcpInfo,
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = state.port is Constraint.Any,
                    onCellClicked = { onObfuscationPortSelected(Constraint.Any) },
                    testTag = UDP_OVER_TCP_PORT_ITEM_AUTOMATIC_TEST_TAG,
                )
            }
            UDP2TCP_PRESET_PORTS.forEach { port ->
                itemWithDivider {
                    SelectableCell(
                        title = port.toString(),
                        isSelected = state.port.getOrNull() == port,
                        onCellClicked = { onObfuscationPortSelected(Constraint.Only(port)) },
                        testTag = String.format(null, UDP_OVER_TCP_PORT_ITEM_X_TEST_TAG, port.value),
                    )
                }
            }
        }
    }
}
