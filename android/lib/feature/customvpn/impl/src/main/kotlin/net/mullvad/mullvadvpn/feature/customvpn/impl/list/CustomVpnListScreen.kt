package net.mullvad.mullvadvpn.feature.customvpn.impl.list

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
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
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlin.text.appendLine
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customvpn.api.CreateCustomVpnImportConfigurationNavResult
import net.mullvad.mullvadvpn.feature.customvpn.api.CreateCustomVpnNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.CustomTunnelConfigDeletedResult
import net.mullvad.mullvadvpn.feature.customvpn.api.CustomTunnelConfigSavedResult
import net.mullvad.mullvadvpn.feature.customvpn.api.EditCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.impl.R
import net.mullvad.mullvadvpn.lib.common.IsModal
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.lib.common.loadingIsModal
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.ui.component.Accordion
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationAndSwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.FirstBaselineAlignedIconAndText
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListHeader
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.theme.color.warning
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewCustomVpnListScreen(
    @PreviewParameter(CustomVpnListUiStatePreviewParameterProvider::class)
    state: Lc<IsModal, CustomVpnListUiState>,
) {
    AppTheme {
        CustomVpnListScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onEditConfiguration = {},
            onCreateConfiguration = {},
            onToggleCustomTunnel = {},
            onBack = {},
        )
    }
}

@Composable
fun SharedTransitionScope.CustomVpnList(
    navigator: Navigator,
    isModal: Boolean,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val viewModel = koinViewModel<CustomVpnListViewModel> { parametersOf(isModal) }

    val context = LocalContext.current
    val customTunnelDefaultName = stringResource(R.string.custom_tunnels_default_name)
    val openFileLauncher =
        rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
            if (uri != null) {
                viewModel.importConfig(
                    name = customTunnelDefaultName,
                    contentResolver = context.contentResolver,
                    uri = uri,
                )
            }
        }

    val resources = LocalResources.current
    val snackbarHostState = remember { SnackbarHostState() }

    val resultStore = LocalResultStore.current
    resultStore.consumeResult<CreateCustomVpnImportConfigurationNavResult> {
        openFileLauncher.launch(SUPPORTED_WIREGUARD_CONFIG_FILE_TYPES)
    }
    resultStore.consumeResult<CustomTunnelConfigSavedResult> {
        snackbarHostState.showSnackbarImmediately(
            resources.getString(R.string.custom_tunnels_form_saved_success),
        )
    }
    resultStore.consumeResult<CustomTunnelConfigDeletedResult> {
        snackbarHostState.showSnackbarImmediately(
            resources.getString(R.string.custom_tunnels_form_deleted_success),
        )
    }

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            CustomVpnListSideEffect.ConfigurationSaved ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.custom_tunnels_configuration_saved),
                    )
                }
            is CustomVpnListSideEffect.FailedToSave ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(
                                R.string.custom_tunnels_configuration_failed_to_save,
                            ),
                    )
                }
        }
    }

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    CustomVpnListScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        modifier =
            Modifier.sharedBounds(
                rememberSharedContentState(key = FeatureIndicator.CUSTOM_VPN),
                animatedVisibilityScope = animatedVisibilityScope,
            ),
        onCreateConfiguration = dropUnlessResumed { navigator.navigate(CreateCustomVpnNavKey) },
        onEditConfiguration =
            dropUnlessResumed { navigator.navigate(EditCustomTunnelNavKey(new = false)) },
        onToggleCustomTunnel = viewModel::onToggleCustomTunnel,
        onBack = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun CustomVpnListScreen(
    state: Lc<IsModal, CustomVpnListUiState>,
    snackbarHostState: SnackbarHostState,
    modifier: Modifier = Modifier,
    onCreateConfiguration: () -> Unit,
    onEditConfiguration: () -> Unit,
    onToggleCustomTunnel: (CustomVpnConfig) -> Unit,
    onBack: () -> Unit,
) {
    var fabHeight by remember { mutableIntStateOf(0) }
    val bottomMargin = with(LocalDensity.current) { fabHeight.toDp() + Dimens.fabSpacing }
    ScaffoldWithSmallTopBar(
        modifier = modifier,
        snackbarHostState = snackbarHostState,
        appBarTitle = stringResource(R.string.custom_tunnels),
        navigationIcon = {
            if (state.contentOrNull()?.isModal ?: state.loadingIsModal) {
                NavigateCloseIconButton(onNavigateClose = onBack)
            } else {
                unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBack) }
            }
        },
        floatingActionButton = {
            if (state is Lc.Content && state.value.customVpnConfig == null) {
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
            }
        },
    ) { contentModifier ->
        val scrollState = rememberScrollState()
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier =
                contentModifier
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .verticalScroll(state = scrollState)
                    .padding(horizontal = Dimens.sideMarginNew)
                    .padding(bottom = bottomMargin),
        ) {
            Description()
            AdvancedFeatureWarning()

            ConfigurationsHeader()

            when (state) {
                is Lc.Loading -> {
                    Loading()
                }
                is Lc.Content -> {
                    CustomVpnListContent(
                        customVpnEnabled = state.value.customVpnConfig?.enabled ?: false,
                        customVpnConfig = state.value.customVpnConfig,
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
                appendLine(stringResource(R.string.custom_tunnels_description_first_paragraph))
                appendLine()
                append(stringResource(R.string.custom_tunnels_description_second_paragraph))
            },
    )
    FirstBaselineAlignedIconAndText(
        modifier = Modifier.padding(bottom = Dimens.mediumPadding),
        text = stringResource(R.string.custom_tunnels_description_info),
        icon = Icons.Outlined.Info,
        iconSize = Dimens.smallishIconSize,
        iconTint = MaterialTheme.colorScheme.onSurface,
        iconToTextPadding = Dimens.smallSpacer,
        shiftIconDown = true,
    )
}

@Composable
private fun AdvancedFeatureWarning() {
    var expandedState by rememberSaveable { mutableStateOf(false) }
    Accordion(
        title = stringResource(R.string.advanced_feature),
        expandedText = annotatedStringResource(R.string.custom_tunnels_advanced_warning),
        icon = Icons.Outlined.Info,
        iconTint = MaterialTheme.colorScheme.warning,
        isExpanded = expandedState,
        onClick = { expandedState = !expandedState },
    )
}

@Composable
private fun ConfigurationsHeader() {
    ListHeader(text = stringResource(R.string.configurations))
}

@Composable
private fun CustomVpnListContent(
    customVpnConfig: CustomVpnConfig?,
    customVpnEnabled: Boolean,
    tunnelStats: TunnelStats?,
    onEditConfiguration: () -> Unit,
    onToggleCustomTunnel: (CustomVpnConfig) -> Unit,
) {
    if (customVpnConfig != null) {
        val dataRx = tunnelStats?.rx ?: 0
        val dataTx = tunnelStats?.tx ?: 0

        NavigationAndSwitchListItem(
            modifier = Modifier.fillMaxWidth(),
            title = customVpnConfig.name ?: stringResource(R.string.custom_tunnels_default_name),
            subtitle =
                stringResource(
                    R.string.rx_tx,
                    dataRx.asFormattedSize(),
                    dataTx.asFormattedSize(),
                ),
            isToggled = customVpnEnabled,
            onCheckedChanged = { onToggleCustomTunnel(customVpnConfig) },
            onNavigationClicked = onEditConfiguration,
        )
        Text(
            modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
            text = annotatedStringResource(R.string.custom_tunnels_single_configuration_warning),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
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
private fun Long.asFormattedSize(): String =
    when {
        this >= GB -> stringResource(R.string.formatted_size_gb, toFloat() / GB)
        this >= MB -> stringResource(R.string.formatted_size_mb, toFloat() / MB)
        this >= KB -> stringResource(R.string.formatted_size_kb, toFloat() / KB)
        else -> stringResource(R.string.formatted_size_b, toFloat())
    }

private const val KB = 1024
private const val MB = KB * 1024
private const val GB = MB * 1024

private val SUPPORTED_WIREGUARD_CONFIG_FILE_TYPES = arrayOf("*/*")
