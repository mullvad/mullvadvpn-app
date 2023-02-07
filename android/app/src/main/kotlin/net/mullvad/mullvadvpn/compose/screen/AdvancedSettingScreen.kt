package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.Divider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import me.onebone.toolbar.CollapsingToolbarScaffold
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.CustomDnsCellSubtitle
import net.mullvad.mullvadvpn.compose.component.CustomDnsComposeCell
import net.mullvad.mullvadvpn.compose.component.DnsCell
import net.mullvad.mullvadvpn.compose.component.DnsCellUiState
import net.mullvad.mullvadvpn.compose.component.MtuComposeCell
import net.mullvad.mullvadvpn.compose.component.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.component.ShowConfirmLocalDnsScreen
import net.mullvad.mullvadvpn.compose.theme.CollapsingToolbarTheme
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.viewmodel.AdvancedSettingUiState

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun AdvancedSettingScreen(
    uiState: AdvancedSettingUiState,
    onToggleCustomDns: (Boolean) -> Unit,
    onNavigateCellClicked: () -> Unit,
    onBackClick: () -> Unit,
) {

    if (uiState is AdvancedSettingUiState.InsertLocalDns) {
        ShowConfirmLocalDnsScreen(
            onConfirm = { uiState.onConfirm() },
            onDismiss = { uiState.onCancel() }
        )
    }

    var cellHeight = dimensionResource(id = R.dimen.cell_height)
    var cellInnerSpacing = dimensionResource(id = R.dimen.cell_inner_spacing)
    var cellVerticalSpacing = dimensionResource(id = R.dimen.cell_label_vertical_padding)
    var cellHorizontalSpacing = dimensionResource(id = R.dimen.cell_left_padding)
    //
    //
    //
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress

    val nestedScrollConnection = remember {
        object : NestedScrollConnection {
            override suspend fun onPostFling(
                consumed: Velocity,
                available: Velocity
            ): Velocity {
                return super.onPostFling(consumed, available)
            }

            override suspend fun onPreFling(available: Velocity): Velocity {
                return super.onPreFling(available)
            }
        }
    }

    var enabled by remember { mutableStateOf(true) }

    var verticalSpacing = dimensionResource(id = R.dimen.vertical_space)
    var cellSideSpacing = dimensionResource(id = R.dimen.cell_left_padding)
    //
    //
    //

    CollapsingToolbarTheme {

        val state = rememberCollapsingToolbarScaffoldState()
        val progress = state.toolbarState.progress

        CollapsingToolbarScaffold(
            modifier = Modifier
                .background(MullvadDarkBlue)
                .fillMaxSize(),
            state = state,
            scrollStrategy = ScrollStrategy.ExitUntilCollapsed,
            enabled = true,
            toolbar = {

                var scaffoldModifier = Modifier
                    .road(
                        whenCollapsed = Alignment.TopCenter,
                        whenExpanded = Alignment.BottomStart
                    )

                CollapsingTopBar(
                    backgroundColor = MullvadDarkBlue,
                    onBackClicked = {
//                                customDnsAdapter?.stopEditing()
                        onBackClick()
                    },
                    title = stringResource(id = R.string.settings_advanced),
                    progress = progress,
                    scaffoldModifier = scaffoldModifier,
                    backTitle = stringResource(id = R.string.settings),
                )
            }
        ) {
            LazyColumn(
                modifier = Modifier
                    .wrapContentHeight()
                    .fillMaxWidth()
                    .animateContentSize()

            ) {
                item {
                    ConstraintLayout(
                        modifier = Modifier
                            .fillMaxWidth()
                            .wrapContentHeight()
                            .padding(top = 8.dp)
                            .background(MullvadBlue)
                    ) {
//                        MtuComposeCell(uiState.mtu, { newMtuValue ->
//                            viewModel.onMtuChanged(newMtuValue)
//                        }, { viewModel.onSubmitMtu() })
                        MtuComposeCell(uiState.mtu, {}, {})
                    }
                }

                item {
                    ConstraintLayout(
                        modifier = Modifier
                            .fillMaxWidth()
                            .wrapContentHeight()
                            .background(MullvadBlue)
                    ) {
                        NavigationComposeCell(
                            title = stringResource(id = R.string.split_tunneling),
                            onClick = {
                                onNavigateCellClicked.invoke()
                            }
                        )
                    }
                }

                item {
                    Spacer(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(22.dp)
                    )
                }

                item {
                    CustomDnsComposeCell(
                        isEnabled = uiState.isCustomDnsEnabled,
                        onToggle = { newValue ->
                            onToggleCustomDns(newValue)
                        }
                    )
                }

                if (uiState.isCustomDnsEnabled) {
                    items(items = uiState.customDnsList) {
                        DnsCell(
                            DnsCellUiState(it),
                            { /*it.removeAt(index)*/ },
                            {},
                            Modifier.animateItemPlacement()
                        )
                        Divider()
                    }
                    item {
                        DnsCell(DnsCellUiState())
                        Divider()
                    }
                }

                item {
                    CustomDnsCellSubtitle(
                        Modifier
                            .background(MullvadDarkBlue)
                            .padding(
                                start = cellHorizontalSpacing,
                                top = 6.dp,
                                end = cellHorizontalSpacing,
                                bottom = cellVerticalSpacing
                            )
                    )
                }
            }
        }
    }
}
