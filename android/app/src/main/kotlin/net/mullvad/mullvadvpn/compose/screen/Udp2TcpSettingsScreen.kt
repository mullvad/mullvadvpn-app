package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.UdpOverTcpPortInfoDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.preview.Udp2TcpSettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.Udp2TcpSettingsUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.constant.UDP2TCP_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.UDP_OVER_TCP_PORT_ITEM_AUTOMATIC_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.UDP_OVER_TCP_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.Udp2TcpSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|Automatic|80")
@Composable
private fun PreviewUdp2TcpSettingsScreen(
    @PreviewParameter(Udp2TcpSettingsUiStatePreviewParameterProvider::class)
    state: Lc<Unit, Udp2TcpSettingsUiState>
) {
    AppTheme {
        Udp2TcpSettingsScreen(
            state = state,
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
    state: Lc<Unit, Udp2TcpSettingsUiState>,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
    navigateUdp2TcpInfo: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.udp_over_tcp),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
            state = lazyListState,
        ) {
            when (state) {
                is Lc.Loading -> loading()
                is Lc.Content ->
                    content(
                        state = state.value,
                        onObfuscationPortSelected = onObfuscationPortSelected,
                        navigateUdp2TcpInfo = navigateUdp2TcpInfo,
                    )
            }
        }
    }
}

private fun LazyListScope.content(
    state: Udp2TcpSettingsUiState,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
    navigateUdp2TcpInfo: () -> Unit,
) {
    itemWithDivider {
        InfoListItem(
            position = Position.Top,
            title = stringResource(R.string.port),
            onInfoClicked = navigateUdp2TcpInfo,
            onCellClicked = navigateUdp2TcpInfo,
        )
    }
    itemWithDivider {
        SelectableListItem(
            hierarchy = Hierarchy.Child1,
            position = Position.Middle,
            title = stringResource(id = R.string.automatic),
            isSelected = state.port is Constraint.Any,
            onClick = { onObfuscationPortSelected(Constraint.Any) },
            testTag = UDP_OVER_TCP_PORT_ITEM_AUTOMATIC_TEST_TAG,
        )
    }
    UDP2TCP_PRESET_PORTS.forEachIndexed { index, port ->
        itemWithDivider {
            SelectableListItem(
                hierarchy = Hierarchy.Child1,
                position =
                    if (index == UDP2TCP_PRESET_PORTS.lastIndex) Position.Bottom
                    else Position.Middle,
                title = port.toString(),
                isSelected = state.port.getOrNull() == port,
                onClick = { onObfuscationPortSelected(Constraint.Only(port)) },
                testTag = String.format(null, UDP_OVER_TCP_PORT_ITEM_X_TEST_TAG, port.value),
            )
        }
    }
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorLarge() }
}
