package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CollapsableAwareToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.RelayLocation
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem

@Preview
@Composable
fun PreviewSelectLocationScreen() {
    val state =
        SelectLocationUiState.ShowData(
            countries = listOf(RelayCountry("Country 1", "Code 1", false, emptyList())),
            selectedRelay = null
        )
    AppTheme { SelectLocationScreen(uiState = state, closeFlow = MutableSharedFlow()) }
}

@Composable
fun SelectLocationScreen(
    uiState: SelectLocationUiState,
    closeFlow: SharedFlow<Unit>,
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress
    LaunchedEffect(Unit) { closeFlow.collect { onBackClick() } }
    CollapsableAwareToolbarScaffold(
        backgroundColor = MaterialTheme.colorScheme.background,
        modifier = Modifier.fillMaxSize(),
        state = state,
        scrollStrategy = ScrollStrategy.ExitUntilCollapsed,
        isEnabledWhenCollapsable = true,
        toolbar = {
            val scaffoldModifier =
                Modifier.road(
                    whenCollapsed = Alignment.TopCenter,
                    whenExpanded = Alignment.BottomStart
                )
            CollapsingTopBar(
                backgroundColor = MaterialTheme.colorScheme.background,
                onBackClicked = { onBackClick() },
                title = stringResource(id = R.string.switch_location),
                progress = progress,
                modifier = scaffoldModifier,
                backTitle = "",
                backIcon = R.drawable.icon_close
            )
        }
    ) {
        LazyColumn {
            item(contentType = ContentType.DESCRIPTION) {
                Text(
                    text = stringResource(id = R.string.select_location_description),
                    fontSize = 13.sp,
                    color = MullvadWhite60,
                    modifier = Modifier.padding(horizontal = Dimens.sideMargin)
                )
            }
            item(contentType = ContentType.SPACER) {
                Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))
            }
            when (uiState) {
                SelectLocationUiState.Loading -> {
                    item(contentType = ContentType.PROGRESS) {
                        CircularProgressIndicator(
                            color = MaterialTheme.colorScheme.onBackground,
                            modifier =
                                Modifier.size(
                                    width = Dimens.progressIndicatorSize,
                                    height = Dimens.progressIndicatorSize
                                )
                        )
                    }
                }
                is SelectLocationUiState.ShowData -> {
                    items(
                        count = uiState.countries.size,
                        key = { index -> uiState.countries[index].code },
                        contentType = { ContentType.ITEM }
                    ) { index ->
                        val country = uiState.countries[index]
                        RelayLocation(
                            relay = country,
                            selectedItem = uiState.selectedRelay,
                            onSelectRelay = onSelectRelay,
                            modifier = Modifier.animateContentSize()
                        )
                    }
                }
            }
        }
    }
}
