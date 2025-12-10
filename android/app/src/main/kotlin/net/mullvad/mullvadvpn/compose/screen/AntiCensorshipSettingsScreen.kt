package net.mullvad.mullvadvpn.compose.screen

import android.os.Parcelable
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.SelectPortDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.preview.AntiCensorshipUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.AntiCensorshipSettingsUiState
import net.mullvad.mullvadvpn.compose.state.ObfuscationSettingItem
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.annotatedStringResource
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.PortType
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ObfuscationModeListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.AntiCensorshipSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Udp2Tcp|Loading")
@Composable
private fun PreviewAntiCensorshipSettingsScreen(
    @PreviewParameter(AntiCensorshipUiStatePreviewParameterProvider::class)
    state: Lc<Unit, AntiCensorshipSettingsUiState>
) {
    AppTheme {
        AntiCensorshipSettingsScreen(
            state = state,
            navigateToShadowSocksSettings = {},
            navigateToUdp2TcpSettings = {},
            onBackClick = {},
            onSelectObfuscationMode = {},
            navigateToWireguardPortSettings = {},
        )
    }
}

@Parcelize data class AntiCensorshipSettingsNavArgs(val isModal: Boolean = false) : Parcelable

@OptIn(ExperimentalSharedTransitionApi::class)
@Destination<RootGraph>(
    style = SlideInFromRightTransition::class,
    navArgs = AntiCensorshipSettingsNavArgs::class,
)
@Composable
fun AntiCensorshipSettings(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<AntiCensorshipSettingsViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    AntiCensorshipSettingsScreen(
        state = state,
        navigateToShadowSocksSettings =
            dropUnlessResumed { navigator.navigate(SelectPortDestination(PortType.Shadowsocks)) },
        navigateToUdp2TcpSettings =
            dropUnlessResumed { navigator.navigate(SelectPortDestination(PortType.Udp2Tcp)) },
        navigateToWireguardPortSettings =
            dropUnlessResumed { navigator.navigate(SelectPortDestination(PortType.Wireguard)) },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onSelectObfuscationMode = viewModel::onSelectObfuscationMode,
    )
}

@Composable
fun AntiCensorshipSettingsScreen(
    state: Lc<Unit, AntiCensorshipSettingsUiState>,
    navigateToShadowSocksSettings: () -> Unit,
    navigateToUdp2TcpSettings: () -> Unit,
    navigateToWireguardPortSettings: () -> Unit,
    onBackClick: () -> Unit,
    onSelectObfuscationMode: (obfuscationMode: ObfuscationMode) -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.anti_censorship),
        navigationIcon = {
            if (state.contentOrNull()?.isModal == true) {
                NavigateCloseIconButton(onBackClick)
            } else {
                NavigateBackIconButton(onNavigateBack = onBackClick)
            }
        },
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier =
                modifier
                    .testTag(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)
                    .padding(horizontal = Dimens.sideMarginNew),
            horizontalAlignment = Alignment.CenterHorizontally,
            state = lazyListState,
        ) {
            when (state) {
                is Lc.Loading -> loading()
                is Lc.Content ->
                    content(
                        state = state.value,
                        navigateToShadowSocksSettings = navigateToShadowSocksSettings,
                        navigateToUdp2TcpSettings = navigateToUdp2TcpSettings,
                        onSelectObfuscationMode = onSelectObfuscationMode,
                        navigateToWireguardPortSettings = navigateToWireguardPortSettings,
                    )
            }
        }
    }
}

@Suppress("LongMethod")
private fun LazyListScope.content(
    state: AntiCensorshipSettingsUiState,
    navigateToShadowSocksSettings: () -> Unit,
    navigateToUdp2TcpSettings: () -> Unit,
    navigateToWireguardPortSettings: () -> Unit,
    onSelectObfuscationMode: (obfuscationMode: ObfuscationMode) -> Unit,
) {
    item {
        Column {
            Text(
                stringResource(R.string.anti_censorship_info_first_paragraph) + "\n",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                annotatedStringResource(R.string.anti_censorship_info_second_paragraph),
                modifier = Modifier.padding(bottom = Dimens.mediumPadding),
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
    itemWithDivider {
        InfoListItem(position = Position.Top, title = stringResource(R.string.method))
    }
    state.items.forEach {
        when (it) {
            is ObfuscationSettingItem.Obfuscation.Automatic ->
                item(key = it::class.simpleName) {
                    SelectableListItem(
                        hierarchy = Hierarchy.Child1,
                        position = Position.Middle,
                        title = stringResource(id = R.string.automatic),
                        isSelected = it.selected,
                        onClick = { onSelectObfuscationMode(ObfuscationMode.Auto) },
                    )
                }
            is ObfuscationSettingItem.Obfuscation.WireguardPort ->
                item(key = it::class.simpleName) {
                    ObfuscationModeListItem(
                        hierarchy = Hierarchy.Child1,
                        position = Position.Middle,
                        obfuscationMode = ObfuscationMode.WireguardPort,
                        isSelected = it.selected,
                        port = it.port,
                        onSelected = { onSelectObfuscationMode(ObfuscationMode.WireguardPort) },
                        onNavigate = navigateToWireguardPortSettings,
                        testTag = WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG,
                    )
                }
            is ObfuscationSettingItem.Obfuscation.Lwo ->
                item(key = it::class.simpleName) {
                    SelectableListItem(
                        hierarchy = Hierarchy.Child1,
                        position = Position.Middle,
                        title = stringResource(id = R.string.lwo),
                        isSelected = it.selected,
                        onClick = { onSelectObfuscationMode(ObfuscationMode.Lwo) },
                        testTag = WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG,
                    )
                }
            is ObfuscationSettingItem.Obfuscation.Quic ->
                item(key = it::class.simpleName) {
                    SelectableListItem(
                        hierarchy = Hierarchy.Child1,
                        position = Position.Middle,
                        title = stringResource(id = R.string.quic),
                        isSelected = it.selected,
                        onClick = { onSelectObfuscationMode(ObfuscationMode.Quic) },
                        testTag = WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG,
                    )
                }
            is ObfuscationSettingItem.Obfuscation.Shadowsocks ->
                item(key = it::class.simpleName) {
                    ObfuscationModeListItem(
                        hierarchy = Hierarchy.Child1,
                        position = Position.Middle,
                        obfuscationMode = ObfuscationMode.Shadowsocks,
                        isSelected = it.selected,
                        port = it.port,
                        onSelected = { onSelectObfuscationMode(ObfuscationMode.Shadowsocks) },
                        onNavigate = navigateToShadowSocksSettings,
                        testTag = WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG,
                    )
                }

            is ObfuscationSettingItem.Obfuscation.UdpOverTcp ->
                item(key = it::class.simpleName) {
                    ObfuscationModeListItem(
                        hierarchy = Hierarchy.Child1,
                        position = Position.Middle,
                        obfuscationMode = ObfuscationMode.Udp2Tcp,
                        isSelected = it.selected,
                        port = it.port,
                        onSelected = { onSelectObfuscationMode(ObfuscationMode.Udp2Tcp) },
                        onNavigate = navigateToUdp2TcpSettings,
                        testTag = WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG,
                    )
                }
            is ObfuscationSettingItem.Obfuscation.Off ->
                item(key = it::class.simpleName) {
                    SelectableListItem(
                        hierarchy = Hierarchy.Child1,
                        position = Position.Bottom,
                        title = stringResource(id = R.string.none),
                        isSelected = it.selected,
                        onClick = { onSelectObfuscationMode(ObfuscationMode.Off) },
                        testTag = WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG,
                    )
                }
            ObfuscationSettingItem.Divider -> {
                item(contentType = it::class.simpleName) {
                    HorizontalDivider(color = Color.Transparent)
                }
            }
        }
    }
    item { Spacer(Modifier.height(Dimens.screenBottomMarginNew)) }
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorLarge() }
}
