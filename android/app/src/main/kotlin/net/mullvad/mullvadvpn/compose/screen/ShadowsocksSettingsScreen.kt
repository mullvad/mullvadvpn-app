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
import com.ramcosta.composedestinations.generated.destinations.ShadowsocksCustomPortDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.dialog.CustomPortNavArgs
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.preview.ShadowsocksSettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_AVAILABLE_PORTS
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.component.listitem.CustomPortListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SHADOWSOCKS_PORT_ITEM_AUTOMATIC_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SHADOWSOCKS_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.ShadowsocksSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|Automatic|Custom")
@Composable
private fun PreviewShadowsocksSettingsScreen(
    @PreviewParameter(ShadowsocksSettingsUiStatePreviewParameterProvider::class)
    state: Lc<Unit, ShadowsocksSettingsUiState>
) {
    AppTheme {
        ShadowsocksSettingsScreen(
            state = state,
            navigateToCustomPortDialog = {},
            onObfuscationPortSelected = {},
            onBackClick = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun ShadowsocksSettings(
    navigator: DestinationsNavigator,
    customPortResult: ResultRecipient<ShadowsocksCustomPortDestination, Port?>,
) {
    val viewModel = koinViewModel<ShadowsocksSettingsViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    customPortResult.OnNavResultValue { port ->
        if (port != null) {
            viewModel.onObfuscationPortSelected(Constraint.Only(port))
        } else {
            viewModel.resetCustomPort()
        }
    }

    ShadowsocksSettingsScreen(
        state = state,
        navigateToCustomPortDialog =
            dropUnlessResumed { customPort ->
                navigator.navigate(
                    ShadowsocksCustomPortDestination(
                        CustomPortNavArgs(
                            customPort = customPort,
                            allowedPortRanges = SHADOWSOCKS_AVAILABLE_PORTS,
                        )
                    )
                )
            },
        onObfuscationPortSelected = viewModel::onObfuscationPortSelected,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun ShadowsocksSettingsScreen(
    state: Lc<Unit, ShadowsocksSettingsUiState>,
    navigateToCustomPortDialog: (customPort: Port?) -> Unit,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.shadowsocks),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
            state = lazyListState,
        ) {
            when (state) {
                is Lc.Loading -> {
                    loading()
                }
                is Lc.Content -> {
                    content(
                        state = state.value,
                        navigateToCustomPortDialog = navigateToCustomPortDialog,
                        onObfuscationPortSelected = onObfuscationPortSelected,
                    )
                }
            }
        }
    }
}

private fun LazyListScope.content(
    state: ShadowsocksSettingsUiState,
    navigateToCustomPortDialog: (Port?) -> Unit,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
) {
    itemWithDivider { InfoListItem(position = Position.Top, title = stringResource(R.string.port)) }
    itemWithDivider {
        SelectableListItem(
            hierarchy = Hierarchy.Child1,
            position = Position.Middle,
            title = stringResource(id = R.string.automatic),
            isSelected = state.port is Constraint.Any,
            onClick = { onObfuscationPortSelected(Constraint.Any) },
            testTag = SHADOWSOCKS_PORT_ITEM_AUTOMATIC_TEST_TAG,
        )
    }
    SHADOWSOCKS_PRESET_PORTS.forEach { port ->
        itemWithDivider {
            SelectableListItem(
                hierarchy = Hierarchy.Child1,
                position = Position.Middle,
                title = port.toString(),
                isSelected = state.port.getOrNull() == port,
                onClick = { onObfuscationPortSelected(Constraint.Only(port)) },
                testTag = String.format(null, SHADOWSOCKS_PORT_ITEM_X_TEST_TAG, port.value),
            )
        }
    }
    itemWithDivider {
        CustomPortListItem(
            hierarchy = Hierarchy.Child1,
            position = Position.Bottom,
            title = stringResource(id = R.string.wireguard_custon_port_title),
            isSelected = state.isCustom,
            port = state.customPort,
            onMainCellClicked = {
                if (state.customPort != null) {
                    onObfuscationPortSelected(Constraint.Only(state.customPort))
                } else {
                    navigateToCustomPortDialog(null)
                }
            },
            onPortCellClicked = { navigateToCustomPortDialog(state.customPort) },
            mainTestTag = SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG,
        )
    }
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorLarge() }
}
