package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment.Companion.CenterHorizontally
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.listitem.AppIconAndTitleListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListHeader
import net.mullvad.mullvadvpn.repository.AppObfuscation
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.AppearanceUiState
import net.mullvad.mullvadvpn.viewmodel.AppearanceViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewAppObfusctionScreen() {
    AppTheme {
        AppearanceScreen(
            uiState =
                Lc.Content(
                    AppearanceUiState(
                        availableObfuscations = AppObfuscation.entries,
                        currentAppObfuscation = AppObfuscation.DEFAULT,
                    )
                )
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun Appearance(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<AppearanceViewModel>()
    val uiState = viewModel.uiState.collectAsStateWithLifecycle().value
    AppearanceScreen(
        uiState = uiState,
        onObfuscationSelected = viewModel::setAppObfuscation,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun AppearanceScreen(
    uiState: Lc<Unit, AppearanceUiState>,
    onObfuscationSelected: (AppObfuscation) -> Unit = {},
    onBackClick: () -> Unit = {},
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.appearance),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
            horizontalAlignment = CenterHorizontally,
            state = lazyListState,
        ) {
            when (uiState) {
                is Lc.Content -> content(uiState.value, onObfuscationSelected)
                is Lc.Loading -> loading()
            }
        }
    }
}

private fun LazyListScope.content(
    uiState: AppearanceUiState,
    onObfuscationSelected: (AppObfuscation) -> Unit = {},
) {
    item { RelayListHeader(content = { Text(text = stringResource(R.string.icon_and_title)) }) }
    itemsIndexedWithDivider(
        items = uiState.availableObfuscations,
        key = { _, item -> item.path },
    ) { index, item ->
        AppIconAndTitleListItem(
            appTitle = stringResource(item.labelId),
            appIcon = item.iconId,
            isSelected = item == uiState.currentAppObfuscation,
            onClick = { onObfuscationSelected(item) },
            position =
                when (index) {
                    0 -> Position.Top
                    uiState.availableObfuscations.lastIndex -> Position.Bottom
                    else -> Position.Middle
                },
        )
    }
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorMedium() }
}
