package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment.Companion.CenterHorizontally
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.listitem.AppIconAndTitleListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.Description
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListHeader
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
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
                ),
            snackbarHostState = SnackbarHostState(),
            onObfuscationSelected = {},
            onBackClick = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun Appearance(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<AppearanceViewModel>()
    val uiState = viewModel.uiState.collectAsStateWithLifecycle().value

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
    LaunchedEffect(uiState.contentOrNull()?.applyingChange) {
        if (uiState.contentOrNull()?.applyingChange == true) {
            launch {
                snackbarHostState.showSnackbarImmediately(
                    message = resources.getString(R.string.applying_changes),
                    duration = SnackbarDuration.Indefinite,
                )
            }
        }
    }

    AppearanceScreen(
        uiState = uiState,
        snackbarHostState = snackbarHostState,
        onObfuscationSelected = viewModel::setAppObfuscation,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun AppearanceScreen(
    uiState: Lc<Unit, AppearanceUiState>,
    snackbarHostState: SnackbarHostState,
    onObfuscationSelected: (AppObfuscation) -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        snackbarHostState = snackbarHostState,
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
    description()
    item { ListHeader(content = { Text(text = stringResource(R.string.icon_and_title)) }) }
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

private fun LazyListScope.description() {
    item {
        Description(
            text =
                buildAnnotatedString {
                    appendLine(stringResource(R.string.appearance_description))
                    appendLine(stringResource(R.string.appearance_description_warning))
                },
            modifier = Modifier.padding(bottom = Dimens.mediumPadding),
        )
    }
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorMedium() }
}
