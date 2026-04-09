package net.mullvad.mullvadvpn.feature.appearance.impl

import android.os.Build
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.assureHasDetailPane
import net.mullvad.mullvadvpn.common.compose.itemWithDivider
import net.mullvad.mullvadvpn.common.compose.navigateReplaceIfDetailPane
import net.mullvad.mullvadvpn.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.appearance.api.AppearanceNavKey
import net.mullvad.mullvadvpn.feature.appicon.api.AppIconNavKey
import net.mullvad.mullvadvpn.feature.language.api.LanguageNavKey
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewAppearanceScreen() {
    AppTheme { AppearanceScreen(onAppIconClick = {}, onLanguageClick = {}, onBackClick = {}) }
}

@Composable
fun Appearance(navigator: Navigator) {
    AppearanceScreen(
        onAppIconClick = dropUnlessResumed { navigator.navigate(AppIconNavKey) },
        onLanguageClick =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                dropUnlessResumed { navigator.navigate(LanguageNavKey) }
            } else {
                null
            },
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AppearanceScreen(
    onAppIconClick: () -> Unit,
    onLanguageClick: (() -> Unit)?,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.appearance),
        navigationIcon = { unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBackClick) } },
    ) { modifier, lazyListState: LazyListState ->
        LazyColumn(
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
            state = lazyListState,
        ) {
            itemWithDivider {
                NavigationListItem(
                    title = stringResource(id = R.string.app_icon),
                    onClick = onAppIconClick,
                    position = if (onLanguageClick != null) Position.Top else Position.Single,
                )
            }
            if (onLanguageClick != null) {
                item {
                    NavigationListItem(
                        title = stringResource(id = R.string.language),
                        onClick = onLanguageClick,
                        position = Position.Bottom,
                    )
                }
            }
        }
    }
}
