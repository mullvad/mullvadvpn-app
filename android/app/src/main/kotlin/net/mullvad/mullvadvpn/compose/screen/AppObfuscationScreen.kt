package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.AppObfuscationCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.repository.AppObfuscation
import net.mullvad.mullvadvpn.viewmodel.AppObfuscationUiState
import net.mullvad.mullvadvpn.viewmodel.AppObfuscationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewAppObfusctionScreen() {
    AppTheme {
        AppObfuscationScreen(uiState = AppObfuscationUiState(
            availableObfuscations = AppObfuscation.entries,
            currentAppObfuscation = AppObfuscation.DEFAULT
        ))
    }

}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun AppObfusctation(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<AppObfuscationViewModel>()
    val uiState = viewModel.uiState.collectAsStateWithLifecycle().value
    AppObfuscationScreen(
        uiState = uiState,
        onObfuscationSelected = viewModel::setAppObfuscation,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun AppObfuscationScreen(
    uiState: AppObfuscationUiState,
    onObfuscationSelected: (AppObfuscation) -> Unit = {},
    onBackClick: () -> Unit = {},
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.app_obfuscation),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(modifier = modifier, state = lazyListState) {
            uiState.availableObfuscations.forEach { obfuscation ->
                item(key = obfuscation.path) {
                    AppObfuscationCell(
                        name = stringResource(obfuscation.labelId),
                        icon = obfuscation.iconId,
                        isSelected = obfuscation == uiState.currentAppObfuscation,
                        onClick = { onObfuscationSelected(obfuscation) },
                    )
                }
            }
        }
    }
}
