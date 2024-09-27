package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
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
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.griditem.AppIconAndTitleGridItem
import net.mullvad.mullvadvpn.lib.ui.component.text.Description
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
    ) { modifier ->
        Column(modifier = modifier.fillMaxSize().padding(horizontal = Dimens.sideMarginNew)) {
            when (uiState) {
                is Lc.Content -> Content(uiState.value, onObfuscationSelected)
                is Lc.Loading -> Loading()
            }
        }
    }
}

@Composable
private fun ColumnScope.Content(
    uiState: AppearanceUiState,
    onObfuscationSelected: (AppObfuscation) -> Unit = {},
) {
    val lazyGridState = rememberLazyGridState()
    Description()
    ListHeader(content = { Text(text = stringResource(R.string.icon_and_title)) })
    LazyVerticalGrid(
        modifier = Modifier.fillMaxWidth().weight(1f),
        state = lazyGridState,
        columns = GridCells.Adaptive(GRID_MIN_WIDTH),
    ) {
        items(items = uiState.availableObfuscations, key = { it.path }) { item ->
            Card(
                shape = MaterialTheme.shapes.large,
                onClick = { onObfuscationSelected(item) },
                colors =
                    CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceDim),
                border =
                    if (item == uiState.currentAppObfuscation) {
                        BorderStroke(
                            width = BORDER_WIDTH,
                            color = MaterialTheme.colorScheme.tertiary,
                        )
                    } else {
                        null
                    },
                modifier = Modifier.padding(all = Dimens.tinyPadding),
            ) {
                AppIconAndTitleGridItem(
                    modifier =
                        Modifier.align(Alignment.CenterHorizontally)
                            .padding(all = Dimens.smallPadding),
                    appTitle = stringResource(item.labelId),
                    appIcon = item.iconId,
                )
            }
        }
    }
}

@Composable
private fun Description() {
    Description(
        text =
            buildAnnotatedString {
                appendLine(stringResource(R.string.appearance_description))
                appendLine()
                append(annotatedStringResource(R.string.appearance_description_warning))
            },
        modifier = Modifier.padding(bottom = Dimens.smallPadding),
    )
}

@Composable
private fun Loading() {
    MullvadCircularProgressIndicatorMedium()
}

private val GRID_MIN_WIDTH = 110.dp
private val BORDER_WIDTH = 3.dp
