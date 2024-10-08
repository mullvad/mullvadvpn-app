package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.MultihopUiState
import net.mullvad.mullvadvpn.viewmodel.MultihopViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
fun PreviewMultihopScreen() {
    AppTheme { MultihopScreen(state = MultihopUiState(false)) }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun Multihop(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<MultihopViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    MultihopScreen(
        state = state,
        onMultihopClick = viewModel::setMultihop,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun MultihopScreen(
    state: MultihopUiState,
    onMultihopClick: (enable: Boolean) -> Unit = {},
    onBackClick: () -> Unit = {},
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.multihop),
        navigationIcon = { NavigateBackIconButton { onBackClick() } },
    ) { modifier ->
        // Multihop image
        Column(modifier = modifier) {
            Description()
            HeaderSwitchComposeCell(
                title = stringResource(R.string.enable),
                isToggled = state.enable,
                onCellClicked = onMultihopClick,
            )
        }
    }
}

@Composable
private fun Description() {
    SwitchComposeSubtitleCell(text = stringResource(R.string.multihop_description))
}
