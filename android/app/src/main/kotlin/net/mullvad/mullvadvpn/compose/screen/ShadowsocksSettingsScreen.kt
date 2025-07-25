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
import com.ramcosta.composedestinations.generated.destinations.ShadowsocksCustomPortDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.CustomPortCell
import net.mullvad.mullvadvpn.compose.cell.InformationComposeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.dialog.CustomPortNavArgs
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_AVAILABLE_PORTS
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.tag.SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SHADOWSOCKS_PORT_ITEM_AUTOMATIC_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SHADOWSOCKS_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.viewmodel.ShadowsocksSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewShadowsocksSettingsScreen() {
    AppTheme {
        ShadowsocksSettingsScreen(
            state = ShadowsocksSettingsState(port = Constraint.Any, validPortRanges = emptyList()),
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
            dropUnlessResumed {
                navigator.navigate(
                    ShadowsocksCustomPortDestination(
                        CustomPortNavArgs(
                            customPort = state.customPort,
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
    state: ShadowsocksSettingsState,
    navigateToCustomPortDialog: () -> Unit,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.shadowsocks),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(modifier = modifier, state = lazyListState) {
            itemWithDivider { InformationComposeCell(title = stringResource(R.string.port)) }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = state.port is Constraint.Any,
                    onCellClicked = { onObfuscationPortSelected(Constraint.Any) },
                    testTag = SHADOWSOCKS_PORT_ITEM_AUTOMATIC_TEST_TAG,
                )
            }
            SHADOWSOCKS_PRESET_PORTS.forEach { port ->
                itemWithDivider {
                    SelectableCell(
                        title = port.toString(),
                        isSelected = state.port.getOrNull() == port,
                        onCellClicked = { onObfuscationPortSelected(Constraint.Only(port)) },
                        testTag = String.format(null, SHADOWSOCKS_PORT_ITEM_X_TEST_TAG, port.value),
                    )
                }
            }
            itemWithDivider {
                CustomPortCell(
                    title = stringResource(id = R.string.wireguard_custon_port_title),
                    isSelected = state.isCustom,
                    port = state.customPort,
                    onMainCellClicked = {
                        if (state.customPort != null) {
                            onObfuscationPortSelected(Constraint.Only(state.customPort))
                        } else {
                            navigateToCustomPortDialog()
                        }
                    },
                    onPortCellClicked = navigateToCustomPortDialog,
                    mainTestTag = SHADOWSOCKS_CUSTOM_PORT_TEXT_TEST_TAG,
                )
            }
        }
    }
}
