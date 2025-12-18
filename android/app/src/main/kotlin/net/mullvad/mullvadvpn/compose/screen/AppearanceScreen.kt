package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyGridScope
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.annotatedStringResource
import net.mullvad.mullvadvpn.compose.util.isTv
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.SPACE_CHAR
import net.mullvad.mullvadvpn.lib.ui.component.griditem.AppIconAndTitleGridItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListHeader
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorMedium
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
            state =
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
        state = uiState,
        snackbarHostState = snackbarHostState,
        onObfuscationSelected = viewModel::setAppObfuscation,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun AppearanceScreen(
    state: Lc<Unit, AppearanceUiState>,
    snackbarHostState: SnackbarHostState,
    onObfuscationSelected: (AppObfuscation) -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        snackbarHostState = snackbarHostState,
        appBarTitle = stringResource(id = R.string.appearance),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyGridState: LazyGridState ->
        LazyVerticalGrid(
            state = lazyGridState,
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
            columns = GridCells.Adaptive(GRID_MIN_WIDTH),
        ) {
            when (state) {
                is Lc.Content -> content(state.value, onObfuscationSelected)
                is Lc.Loading -> loading()
            }
        }
    }
}

private fun LazyGridScope.content(
    state: AppearanceUiState,
    onObfuscationSelected: (AppObfuscation) -> Unit = {},
) {
    item(span = { GridItemSpan(this.maxLineSpan) }) { Description() }
    item(span = { GridItemSpan(this.maxLineSpan) }) {
        ListHeader(content = { Text(text = stringResource(R.string.icon_and_title)) })
    }
    items(items = state.availableObfuscations, key = { it.clazz.name }) { item ->
        Card(
            shape = MaterialTheme.shapes.large,
            onClick = { onObfuscationSelected(item) },
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceDim),
            border =
                if (item == state.currentAppObfuscation) {
                    BorderStroke(width = BORDER_WIDTH, color = MaterialTheme.colorScheme.tertiary)
                } else {
                    null
                },
            modifier = Modifier.padding(all = Dimens.tinyPadding),
        ) {
            AppIconAndTitleGridItem(
                modifier =
                    Modifier.align(Alignment.CenterHorizontally).padding(all = Dimens.smallPadding),
                appTitle = stringResource(item.labelId),
                appIcon =
                    if (isTv()) {
                        item.bannerId
                    } else {
                        item.iconId
                    },
            )
        }
    }
}

@Composable
private fun Description() {
    ScreenDescription(
        text =
            buildAnnotatedString {
                appendLine(stringResource(R.string.appearance_description))
                appendLine()
                append(annotatedStringResource(R.string.appearance_description_warning))
                if(isTv()) {
                    append(SPACE_CHAR)
                    append(stringResource(R.string.appearance_description_warning_tv))
                }
            },
        modifier = Modifier.padding(bottom = Dimens.smallPadding),
    )
}

private fun LazyGridScope.loading() {
    item { MullvadCircularProgressIndicatorMedium() }
}

private val GRID_MIN_WIDTH = 110.dp
private val BORDER_WIDTH = 3.dp
