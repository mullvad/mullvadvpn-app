package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.preview.NotificationSettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.NotificationSettingsUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.NotificationSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|Normal")
@Composable
private fun PreviewNotificationSettingsScreen(
    @PreviewParameter(NotificationSettingsUiStatePreviewParameterProvider::class)
    state: Lc<Unit, NotificationSettingsUiState>
) {
    AppTheme {
        NotificationSettingsScreen(
            state = state,
            onBackClick = {},
            onToggleLocationInNotifications = {},
        )
    }
}

@OptIn(ExperimentalSharedTransitionApi::class)
@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun NotificationSettings(navigator: DestinationsNavigator) {
    val vm = koinViewModel<NotificationSettingsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    NotificationSettingsScreen(
        state = state,
        onBackClick = { navigator.navigateUp() },
        onToggleLocationInNotifications = vm::onToggleLocationInNotifications,
    )
}

@Composable
fun NotificationSettingsScreen(
    state: Lc<Unit, NotificationSettingsUiState>,
    onBackClick: () -> Unit,
    onToggleLocationInNotifications: (Boolean) -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_notifications),
        navigationIcon = { NavigateBackIconButton { onBackClick() } },
    ) { modifier ->
        Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = modifier) {
            when (state) {
                is Lc.Loading -> Loading()
                is Lc.Content -> {
                    NotificationSettingsContent(
                        state = state.value,
                        onToggleLocationInNotifications = onToggleLocationInNotifications,
                    )
                }
            }
        }
    }
}

@Composable
private fun NotificationSettingsContent(
    state: NotificationSettingsUiState,
    onToggleLocationInNotifications: (Boolean) -> Unit,
) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        HeaderSwitchComposeCell(
            title = stringResource(R.string.enable_location_in_notification),
            isToggled = state.locationInNotificationEnabled,
            onCellClicked = onToggleLocationInNotifications,
        )
    }
}

@Composable
private fun Loading() {
    MullvadCircularProgressIndicatorLarge()
}
