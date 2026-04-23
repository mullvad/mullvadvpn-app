package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.list

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customvpn.api.CreateCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.EditCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.impl.R
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.PersonalVpnConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationAndSwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.FirstBaselineAlignedIconAndText
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListHeader
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewCustomTunnelsListScreen(
    @PreviewParameter(CustomTunnelsListUiStatePreviewParameterProvider::class)
    state: Lc<Unit, CustomTunnelsListUiState>
) {
    CustomTunnelsListScreen(
        state = state,
        isModal = false,
        onEditConfiguration = {},
        onCreateConfiguration = {},
        onToggleCustomTunnel = {},
        onBack = {},
    )
}

@Composable
fun SharedTransitionScope.CustomTunnelsList(
    navigator: Navigator,
    isModal: Boolean,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val viewModel = koinViewModel<CustomTunnelsListViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    CustomTunnelsListScreen(
        state = state,
        isModal = isModal,
        modifier =
            Modifier.sharedBounds(
                rememberSharedContentState(key = FeatureIndicator.PERSONAL_VPN),
                animatedVisibilityScope = animatedVisibilityScope,
            ),
        onCreateConfiguration = dropUnlessResumed { navigator.navigate(CreateCustomTunnelNavKey) },
        onEditConfiguration =
            dropUnlessResumed { navigator.navigate(EditCustomTunnelNavKey(new = false)) },
        onToggleCustomTunnel = viewModel::onEnableCustomVpn,
        onBack = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun CustomTunnelsListScreen(
    state: Lc<Unit, CustomTunnelsListUiState>,
    isModal: Boolean,
    modifier: Modifier = Modifier,
    onCreateConfiguration: () -> Unit,
    onEditConfiguration: () -> Unit,
    onToggleCustomTunnel: (Boolean) -> Unit,
    onBack: () -> Unit,
) {
    var fabHeight by remember { mutableIntStateOf(0) }
    val bottomMargin = with(LocalDensity.current) { fabHeight.toDp() + Dimens.fabSpacing }
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(R.string.custom_tunnels),
        navigationIcon = {
            if (isModal) {
                NavigateCloseIconButton(onNavigateClose = onBack)
            } else {
                NavigateBackIconButton(onNavigateBack = onBack)
            }
        },
        floatingActionButton = {
            FloatingActionButton(
                modifier = Modifier.onGloballyPositioned { fabHeight = it.size.height },
                onClick = onCreateConfiguration,
                containerColor = MaterialTheme.colorScheme.tertiaryContainer,
                contentColor = MaterialTheme.colorScheme.onTertiary,
            ) {
                Icon(
                    imageVector = Icons.Rounded.Add,
                    contentDescription = stringResource(id = R.string.add),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        },
    ) { modifier ->
        val scrollState = rememberScrollState()
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier =
                modifier
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .verticalScroll(state = scrollState)
                    .padding(horizontal = Dimens.sideMarginNew)
                    .padding(bottom = bottomMargin),
        ) {
            Description()

            ConfigurationsHeader()

            when (state) {
                is Lc.Loading -> {
                    Loading()
                }
                is Lc.Content -> {
                    CustomTunnelsListContent(
                        customVpnEnabled =
                            state.value.customVpnConfigState is CustomTunnelConfigState.Enabled,
                        customVpnConfig = state.value.customVpnConfigState?.config,
                        tunnelStats = state.value.tunnelStats,
                        onEditConfiguration = onEditConfiguration,
                        onToggleCustomTunnel = onToggleCustomTunnel,
                    )
                }
            }
        }
    }
}

@Composable
private fun ColumnScope.Description() {
    Image(
        contentScale = ContentScale.FillWidth,
        modifier =
            Modifier.widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                .fillMaxWidth()
                .align(Alignment.CenterHorizontally),
        painter = painterResource(R.drawable.custom_tunnels),
        contentDescription = null,
    )
    ScreenDescription(
        modifier = Modifier.padding(vertical = Dimens.smallPadding),
        text =
            buildString {
                appendLine(R.string.custom_tunnels_description_first_paragraph)
                appendLine()
                appendLine(R.string.custom_tunnels_description_second_paragraph)
            },
    )
    FirstBaselineAlignedIconAndText(
        modifier = Modifier.padding(bottom = Dimens.smallPadding),
        text = stringResource(R.string.custom_tunnels_description_info),
        icon = Icons.Outlined.Info,
        iconSize = Dimens.smallishIconSize,
        iconTint = MaterialTheme.colorScheme.onSurfaceVariant,
        iconToTextPadding = Dimens.smallSpacer,
        shiftIconDown = true,
    )
}

@Composable
private fun ConfigurationsHeader() {
    ListHeader(text = stringResource(R.string.configurations))
}

@Composable
private fun CustomTunnelsListContent(
    customVpnConfig: PersonalVpnConfig?,
    customVpnEnabled: Boolean,
    tunnelStats: TunnelStats?,
    onEditConfiguration: () -> Unit,
    onToggleCustomTunnel: (Boolean) -> Unit,
) {
    if (customVpnConfig != null) {
        NavigationAndSwitchListItem(
            modifier = Modifier.fillMaxWidth(),
            title = "Custom tunnel", // TODO
            subtitle =
                tunnelStats?.let {
                    stringResource(
                        R.string.tx_rx,
                        tunnelStats.tx.asFormattedSize(),
                        tunnelStats.rx.asFormattedSize(),
                    )
                },
            isToggled = customVpnEnabled,
            onCellClicked = onToggleCustomTunnel,
            onNavigationClicked = onEditConfiguration,
        )
    } else {
        ScreenDescription(
            modifier = Modifier.padding(horizontal = Dimens.smallPadding),
            text = stringResource(R.string.custom_tunnels_tap_to_add),
        )
    }
}

@Composable
private fun Loading() {
    MullvadCircularProgressIndicatorLarge()
}

@Composable
private fun Long.asFormattedSize(): String {
    val kb = 1024
    val mb = kb * 1024
    val gb = mb * 1024

    return when {
        this >= gb -> stringResource(R.string.formatted_size_gb, toFloat() / gb)
        this >= mb -> stringResource(R.string.formatted_size_mb, toFloat() / mb)
        this >= kb -> stringResource(R.string.formatted_size_kb, toFloat() / kb)
        else -> "${this}B" // TODO
    }
}
