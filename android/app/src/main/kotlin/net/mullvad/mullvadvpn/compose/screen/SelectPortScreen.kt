package net.mullvad.mullvadvpn.compose.screen

import android.os.Parcelable
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
import com.ramcosta.composedestinations.generated.destinations.CustomPortDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.dialog.CustomPortDialogNavArgs
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.preview.SelectPortUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SelectPortUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortType
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.listitem.CustomPortListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_PORT_CUSTOM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_PORT_ITEM_AUTOMATIC_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.SelectPortViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|Automatic|80")
@Composable
private fun PreviewSelectPortScreen(
    @PreviewParameter(SelectPortUiStatePreviewParameterProvider::class)
    state: Lc<Unit, SelectPortUiState>
) {
    AppTheme {
        SelectPortScreen(
            state = state,
            onObfuscationPortSelected = {},
            onBackClick = {},
            navigateToCustomPortDialog = {},
        )
    }
}

@Parcelize data class SelectPortNavArgs(val portType: PortType) : Parcelable

@Destination<RootGraph>(
    style = SlideInFromRightTransition::class,
    navArgs = SelectPortNavArgs::class,
)
@Composable
fun SelectPort(
    navigator: DestinationsNavigator,
    customPortResult: ResultRecipient<CustomPortDestination, Port?>,
) {
    val viewModel = koinViewModel<SelectPortViewModel>()
    val stateLc by viewModel.uiState.collectAsStateWithLifecycle()

    customPortResult.OnNavResultValue { port ->
        if (port != null) {
            viewModel.onPortSelected(Constraint.Only(port))
        } else {
            viewModel.resetCustomPort()
        }
    }

    SelectPortScreen(
        state = stateLc,
        onObfuscationPortSelected = viewModel::onPortSelected,
        navigateToCustomPortDialog =
            dropUnlessResumed { customPort ->
                val state = stateLc.contentOrNull() ?: return@dropUnlessResumed

                navigator.navigate(
                    CustomPortDestination(
                        CustomPortDialogNavArgs(
                            portType = state.portType,
                            allowedPortRanges = state.allowedPortRanges,
                            customPort = customPort,
                        )
                    )
                )
            },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun SelectPortScreen(
    state: Lc<Unit, SelectPortUiState>,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
    navigateToCustomPortDialog: (Port?) -> Unit,
    onBackClick: () -> Unit,
) {

    ScaffoldWithMediumTopBar(
        appBarTitle = state.contentOrNull()?.title ?: "",
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
                        navigateToCustomPortDialog = navigateToCustomPortDialog,
                    )
            }
        }
    }
}

private fun LazyListScope.content(
    state: SelectPortUiState,
    onObfuscationPortSelected: (Constraint<Port>) -> Unit,
    navigateToCustomPortDialog: (Port?) -> Unit,
) {
    itemWithDivider { InfoListItem(position = Position.Top, title = stringResource(R.string.port)) }
    itemWithDivider {
        SelectableListItem(
            hierarchy = Hierarchy.Child1,
            position =
                if (state.customPortEnabled || state.presetPorts.isNotEmpty()) Position.Middle
                else Position.Bottom,
            title = stringResource(id = R.string.automatic),
            isSelected = state.port is Constraint.Any,
            onClick = { onObfuscationPortSelected(Constraint.Any) },
            testTag = SELECT_PORT_ITEM_AUTOMATIC_TEST_TAG,
        )
    }
    state.presetPorts.forEachIndexed { index, port ->
        itemWithDivider {
            SelectableListItem(
                hierarchy = Hierarchy.Child1,
                position =
                    if (state.customPortEnabled || index != state.presetPorts.lastIndex)
                        Position.Middle
                    else Position.Bottom,
                title = port.toString(),
                isSelected = state.port.getOrNull() == port,
                onClick = { onObfuscationPortSelected(Constraint.Only(port)) },
                testTag = SELECT_PORT_ITEM_X_TEST_TAG.format(port.value),
            )
        }
    }
    if (state.customPortEnabled) {
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
                mainTestTag = SELECT_PORT_CUSTOM_TEST_TAG,
            )
        }
    }
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorLarge() }
}
