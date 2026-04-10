package net.mullvad.mullvadvpn.feature.language.impl

import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import java.util.Locale
import net.mullvad.mullvadvpn.common.compose.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewLanguageScreen() {
    AppTheme {
        LanguageScreen(
            state =
                LanguageUiState(
                        languages =
                            listOf(
                                LanguageItem.SystemDefault(isSelected = true),
                                LanguageItem.Language(
                                    locale = Locale.ENGLISH,
                                    displayName = "English",
                                    isSelected = false,
                                ),
                                LanguageItem.Language(
                                    locale = Locale.forLanguageTag("sv"),
                                    displayName = "Svenska",
                                    isSelected = false,
                                ),
                            )
                    )
                    .toLc(),
            onLanguageSelected = {},
            onBackClick = {},
        )
    }
}

@Composable
@RequiresApi(android.os.Build.VERSION_CODES.TIRAMISU)
fun Language(navigator: Navigator) {
    val viewModel = koinViewModel<LanguageViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LanguageScreen(
        state = state,
        onLanguageSelected = viewModel::setLanguage,
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun LanguageScreen(
    state: Lc<Unit, LanguageUiState>,
    onLanguageSelected: (Locale?) -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.language),
        navigationIcon = { unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBackClick) } },
    ) { modifier, lazyListState: LazyListState ->
        LazyColumn(
            state = lazyListState,
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
        ) {
            when (state) {
                is Lc.Loading -> item { MullvadCircularProgressIndicatorLarge() }
                is Lc.Content -> {
                    val languages = state.value.languages
                    itemsIndexedWithDivider(items = languages, key = { _, item -> item.key }) {
                        index,
                        item ->
                        SelectableListItem(
                            title =
                                when (item) {
                                    is LanguageItem.Language -> item.displayName
                                    is LanguageItem.SystemDefault ->
                                        stringResource(R.string.system_default)
                                },
                            isSelected = item.isSelected,
                            onClick = { onLanguageSelected(item.locale) },
                            position =
                                when {
                                    languages.size == 1 -> Position.Single
                                    index == 0 -> Position.Top
                                    index == languages.size - 1 -> Position.Bottom
                                    else -> Position.Middle
                                },
                        )
                    }
                }
            }
        }
    }
}
