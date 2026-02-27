package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@Destination<ExternalModuleGraph>(style = SlideInFromRightTransition::class)
@Composable
fun PersonalVpn(navigator: DestinationsNavigator) {
    val vm = koinViewModel<PersonalVpnViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect -> }

    PersonalVpnScreen(
        state = state,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        vm::onToggle,
        snackbarHostState = snackbarHostState,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PersonalVpnScreen(
    state: Lc<Boolean, PersonalVpnUiState>,
    onBackClick: () -> Unit,
    onTogglePersonalVpn: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.personal_vpn),
        modifier = modifier,
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier ->
        Column(
            modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMarginNew),
            verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
        ) {
            if (state is Lc.Content) {
                ScreenDescription(stringResource(R.string.personal_vpn_description))
                SwitchListItem(
                    title = stringResource(R.string.enable),
                    isToggled = state.value.enabled,
                    onCellClicked = { onTogglePersonalVpn(!state.value.enabled) },
                    position = Position.Single,
                )
                Spacer(modifier = Modifier.weight(1f))
                SnackbarHost(hostState = snackbarHostState) { MullvadSnackbar(snackbarData = it) }
            }
        }
    }
}
