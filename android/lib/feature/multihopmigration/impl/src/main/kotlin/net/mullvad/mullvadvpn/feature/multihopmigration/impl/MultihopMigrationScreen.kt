package net.mullvad.mullvadvpn.feature.multihopmigration.impl

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.multihopmigration.api.MultihopMigrationNavKey
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithNavigationButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryOutlinedButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewMultihopMigrationScreen(
    @PreviewParameter(MultihopMigrationUiStatePreviewParameterProvider::class)
    state: MultihopMigrationUiState
) {
    AppTheme {
        MultihopMigrationScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onCloseClick = {},
            onBackClick = {},
            onNextClick = {},
            onFinishMigration = {},
            onSetEntry = {},
            onSetMultihopMode = {},
        )
    }
}

@Composable
fun MultihopMigration(navKey: MultihopMigrationNavKey, navigator: Navigator) {
    val viewModel = koinViewModel<MultihopMigrationViewModel> { parametersOf(navKey) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val resources = LocalResources.current
    val snackbarHostState = remember { SnackbarHostState() }

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            MultihopMigrationScreenSideEffect.CloseScreen -> navigator.goBack()
            MultihopMigrationScreenSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(
                    message = resources.getString(R.string.error_occurred)
                )
        }
    }
    MultihopMigrationScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onCloseClick = dropUnlessResumed { navigator.goBack() },
        onBackClick = viewModel::previousPage,
        onNextClick = viewModel::nextPage,
        onSetEntry = viewModel::setEntryLocation,
        onSetMultihopMode = viewModel::setMultihopMode,
        onFinishMigration = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun MultihopMigrationScreen(
    state: MultihopMigrationUiState,
    snackbarHostState: SnackbarHostState,
    onCloseClick: () -> Unit,
    onBackClick: () -> Unit,
    onNextClick: () -> Unit,
    onFinishMigration: () -> Unit,
    onSetEntry: (entry: Constraint<RelayItemId>) -> Unit,
    onSetMultihopMode: (MultihopMode) -> Unit,
) {
    BackHandler(
        onBack = {
            if (state.currentPageIndex > 0) {
                onBackClick()
            } else {
                onCloseClick()
            }
        }
    )

    val backgroundColor = MaterialTheme.colorScheme.surface
    ScaffoldWithNavigationButton(
        modifier = Modifier.background(backgroundColor),
        navigationIcon = { NavigateCloseIconButton(onNavigateClose = onCloseClick) },
        bottomBar = {
            BottomBar(
                currentPage = state.currentPageIndex,
                pages = state.size,
                onBackClick = onBackClick,
                onNextClick = onNextClick,
                onFinishMigration = onFinishMigration,
            )
        },
        snackbarHostState = snackbarHostState,
    ) { modifier ->
        val pagerState =
            rememberPagerState(initialPage = state.currentPageIndex, pageCount = { state.size })

        LaunchedEffect(state.currentPageIndex) {
            pagerState.animateScrollToPage(state.currentPageIndex)
        }

        HorizontalPager(
            modifier = modifier,
            state = pagerState,
            userScrollEnabled = false,
            key = { index -> state.multihopMigrationPages[index] },
        ) { page ->
            when (val currentPage = state.multihopMigrationPages[page]) {
                is MultihopMigrationPage.NewMultihopMode ->
                    NewMultihopMode(currentPage.multihopMigrationState)
                MultihopMigrationPage.DirectOnlyRemoved -> DirectOnlyRemoved()
                MultihopMigrationPage.SeparateFilters -> SeparateFilters()
                is MultihopMigrationPage.SuggestedMultihopEntry ->
                    SuggestedMultihopEntry(entry = state.entryLocation, onSetEntry = onSetEntry)
                MultihopMigrationPage.SuggestedAction ->
                    SuggestedAction(onSetMultihopMode = onSetMultihopMode)
                MultihopMigrationPage.EntrySetToAutomatic -> EntrySetToAutomatic()
            }
        }
    }
}

@Composable
private fun NewMultihopMode(multihopMigrationState: MultihopMigrationState) {
    MultihopMigrationPage(title = stringResource(R.string.new_multihop_modes_title)) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            text =
                buildAnnotatedString {
                    appendLine(stringResource(R.string.new_multihop_modes_first_paragraph))
                    appendLine()
                    when (multihopMigrationState) {
                        MultihopMigrationState.ON_TO_ALWAYS -> {
                            MultihopModeDescription(
                                fromModeToMode =
                                    stringResource(R.string.new_multihop_modes_on_to_always),
                                newMode = stringResource(R.string.always),
                                description =
                                    stringResource(R.string.new_multihop_nodes_always_description),
                            )
                        }
                        MultihopMigrationState.OFF_TO_NEVER -> {
                            MultihopModeDescription(
                                fromModeToMode =
                                    stringResource(R.string.new_multihop_modes_off_to_never),
                                newMode = stringResource(R.string.never),
                                description =
                                    stringResource(R.string.new_multihop_nodes_never_description),
                            )
                        }
                        MultihopMigrationState.OFF_TO_WHEN_NEEDED -> {
                            MultihopModeDescription(
                                fromModeToMode =
                                    stringResource(R.string.new_multihop_modes_off_to_when_needed),
                                newMode = stringResource(R.string.when_needed),
                                description =
                                    stringResource(
                                        R.string.new_multihop_nodes_when_needed_description
                                    ),
                            )
                        }
                        MultihopMigrationState.OFF_TO_ALWAYS -> {
                            MultihopModeDescription(
                                fromModeToMode =
                                    stringResource(R.string.new_multihop_modes_off_to_always),
                                newMode = stringResource(R.string.always),
                                description =
                                    stringResource(R.string.new_multihop_nodes_always_description),
                            )
                        }
                    }
                },
        )
    }
}

@Composable
private fun AnnotatedString.Builder.MultihopModeDescription(
    fromModeToMode: String,
    newMode: String,
    description: String,
) {
    withStyle(SpanStyle(color = MaterialTheme.colorScheme.onSurface)) { appendLine(fromModeToMode) }
    appendLine()
    withStyle(
        SpanStyle(color = MaterialTheme.colorScheme.onSurface, fontWeight = FontWeight.Bold)
    ) {
        appendLine(newMode)
    }
    appendLine(description)
}

@Composable
private fun DirectOnlyRemoved() {
    MultihopMigrationPage(title = stringResource(R.string.direct_only_removed_title)) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            text =
                buildString {
                    appendLine(stringResource(R.string.direct_only_removed_first_paragraph))
                    appendLine()
                    appendLine(stringResource(R.string.direct_only_removed_second_paragraph))
                    appendLine()
                    appendLine(stringResource(R.string.direct_only_removed_third_paragraph))
                },
        )
    }
}

@Composable
private fun SeparateFilters() {
    MultihopMigrationPage(title = stringResource(R.string.separate_filters_title)) {
        Image(
            painter = painterResource(R.drawable.filter_migration_illustration),
            contentDescription = null,
        )
        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
        Text(
            modifier = Modifier.fillMaxWidth(),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            text =
                buildAnnotatedString {
                    appendLine(stringResource(R.string.separate_filters_title_first_paragraph))
                    appendLine()
                    withStyle(SpanStyle(color = MaterialTheme.colorScheme.onSurface)) {
                        appendLine(stringResource(R.string.separate_filters_title_second_paragraph))
                    }
                },
        )
    }
}

@Composable
private fun SuggestedMultihopEntry(
    entry: Constraint<RelayItemId>?,
    onSetEntry: (entry: Constraint<RelayItemId>) -> Unit,
) {
    var buttonState by remember { mutableStateOf<ButtonState>(ButtonState.Idle) }
    LaunchedEffect(entry) {
        buttonState = if (entry == Constraint.Any) ButtonState.Done else ButtonState.Idle
    }

    MultihopMigrationPage(title = stringResource(R.string.suggested_multihop_entry_title)) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            text =
                buildAnnotatedString {
                    appendLine(stringResource(R.string.suggested_multihop_entry_first_paragraph))
                    appendLine()
                    appendLine(stringResource(R.string.automatic_entry_description))
                    appendLine()
                    withStyle(SpanStyle(fontWeight = FontWeight.Bold)) {
                        appendLine(stringResource(R.string.suggested_multihop_entry_warning))
                    }
                },
        )
        Spacer(modifier = Modifier.weight(1f))
        PrimaryButton(
            isEnabled = (entry != Constraint.Any) && buttonState != ButtonState.Loading,
            isLoading = buttonState == ButtonState.Loading,
            trailingIcon =
                if (buttonState == ButtonState.Done) {
                    {
                        Icon(
                            imageVector = Icons.Rounded.Check,
                            tint = MaterialTheme.colorScheme.onPrimary,
                            contentDescription = null,
                        )
                    }
                } else {
                    null
                },
            text = stringResource(R.string.suggested_multihop_entry_button),
            onClick = {
                buttonState = ButtonState.Loading
                onSetEntry(Constraint.Any)
            },
        )
    }
}

@Composable
private fun SuggestedAction(onSetMultihopMode: (MultihopMode) -> Unit) {
    MultihopMigrationPage(title = stringResource(R.string.suggested_action_title)) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            text =
                buildAnnotatedString {
                    appendLine(stringResource(R.string.suggested_action_first_paragraph))
                    appendLine()
                    appendLine(stringResource(R.string.suggested_action_second_paragraph))
                    appendLine()
                    withStyle(SpanStyle(fontStyle = FontStyle.Italic)) {
                        appendLine(stringResource(R.string.suggested_action_warning))
                    }
                },
        )
        Spacer(modifier = Modifier.weight(1f))
        PrimaryButton(
            text = stringResource(R.string.suggested_action_button),
            onClick = { onSetMultihopMode(MultihopMode.WHEN_NEEDED) },
        )
    }
}

@Composable
private fun MultihopMigrationPage(title: String, content: @Composable ColumnScope.() -> Unit) {
    val scrollState = rememberScrollState()
    Column(
        modifier =
            Modifier.fillMaxSize()
                .padding(horizontal = Dimens.sideMarginNew)
                .verticalScroll(scrollState)
                .drawVerticalScrollbar(
                    state = scrollState,
                    color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar),
                ),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Icon(
            modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
            imageVector = Icons.Rounded.Info,
            contentDescription = "",
            tint = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
        Text(
            text = title,
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(modifier = Modifier.height(Dimens.mediumPadding))
        content()
    }
}

@Composable
private fun EntrySetToAutomatic() {
    MultihopMigrationPage(title = stringResource(R.string.entry_set_to_automatic_title)) {
        Image(
            painter = painterResource(R.drawable.entry_set_to_automatic_illustration),
            contentDescription = null,
        )
        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
        Text(
            modifier = Modifier.fillMaxWidth(),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            text =
                buildAnnotatedString {
                    appendLine(stringResource(R.string.entry_set_to_automatic_first_paragraph))
                    appendLine()
                    appendLine(stringResource(R.string.automatic_entry_description))
                },
        )
    }
}

@Composable
private fun BottomBar(
    currentPage: Int,
    pages: Int,
    onBackClick: () -> Unit,
    onNextClick: () -> Unit,
    onFinishMigration: () -> Unit,
) {
    Column(
        modifier =
            Modifier.windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom))
                .padding(vertical = Dimens.screenBottomMargin, horizontal = Dimens.sideMarginNew)
    ) {
        if (pages > 1) {
            PageIndicator(currentPage = currentPage, pages = pages)
        }

        BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
            val halfWidth = (maxWidth - Dimens.smallPadding) / 2
            val backButtonWidth by
                animateDpAsState(
                    targetValue = if (currentPage > 0) halfWidth else 0.dp,
                    label = "backButtonWidth",
                )
            val backButtonAlpha =
                if (halfWidth > 0.dp) (backButtonWidth / halfWidth).coerceIn(0f, 1f) else 0f

            Row(modifier = Modifier.fillMaxWidth()) {
                if (backButtonWidth > 0.dp) {
                    PrimaryOutlinedButton(
                        text = stringResource(R.string.back),
                        onClick = onBackClick,
                        modifier = Modifier.width(backButtonWidth).alpha(backButtonAlpha),
                    )
                    Spacer(
                        modifier = Modifier.width((Dimens.smallPadding.value * backButtonAlpha).dp)
                    )
                }
                // TODO Next need arrow icon
                PrimaryButton(
                    text =
                        if (currentPage == pages - 1) stringResource(R.string.got_it)
                        else stringResource(R.string.next),
                    onClick = if (currentPage == pages - 1) onFinishMigration else onNextClick,
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

@Composable
private fun PageIndicator(currentPage: Int, pages: Int) {
    Row(
        Modifier.wrapContentHeight().fillMaxWidth().padding(bottom = Dimens.mediumPadding),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.Bottom,
    ) {
        repeat(pages) { iteration ->
            if (currentPage == iteration) {
                Box(
                    modifier =
                        Modifier.padding(Dimens.indicatorPadding)
                            .clip(CircleShape)
                            .background(MaterialTheme.colorScheme.onPrimary)
                            .height(Dimens.indicatorSize)
                            .width(Dimens.indicatorWidthSelected)
                )
            } else {
                Box(
                    modifier =
                        Modifier.padding(Dimens.indicatorPadding)
                            .clip(CircleShape)
                            .background(MaterialTheme.colorScheme.primary)
                            .size(Dimens.indicatorSize)
                )
            }
        }
    }
}

private sealed interface ButtonState {
    object Idle : ButtonState

    object Loading : ButtonState

    object Done : ButtonState
}
