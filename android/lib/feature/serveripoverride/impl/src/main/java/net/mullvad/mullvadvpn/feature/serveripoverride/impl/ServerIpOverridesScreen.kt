package net.mullvad.mullvadvpn.feature.serveripoverride.impl

import android.content.res.Resources
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverrideByFileNavResult
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverrideByTextNavResult
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverridesNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ResetServerIpOverrideConfirmationNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ResetServerIpOverrideConfirmationNavResult
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ServerIpOverrideInfoNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ServerIpOverrideNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.SettingsPatchError
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ServerIpOverridesListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_IMPORT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_INFO_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaDisabled
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview("Loaded.Active|Loaded.Inactive|Loading")
@Composable
private fun PreviewServerIpOverridesScreen(
    @PreviewParameter(ServerIpOverridesUiStatePreviewParameterProvider::class)
    state: Lc<Boolean, ServerIpOverridesUiState>
) {
    AppTheme {
        ServerIpOverridesScreen(
            state = state,
            onBackClick = {},
            onInfoClick = {},
            onImportClick = {},
            onResetOverridesClick = {},
            snackbarHostState = SnackbarHostState(),
        )
    }
}

@Composable
fun SharedTransitionScope.ServerIpOverrides(
    navArgs: ServerIpOverrideNavKey,
    navigator: Navigator,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val vm = koinViewModel<ServerIpOverridesViewModel> { parametersOf(navArgs) }
    val state by vm.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }

    val resources = LocalResources.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is ServerIpOverridesUiSideEffect.ImportResult ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = sideEffect.error.toString(resources),
                        actionLabel = null,
                    )
                }
        }
    }

    val resultStore = LocalResultStore.current

    val openFileLauncher =
        rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) {
            if (it != null) {
                vm.importFile(it)
            }
        }

    val scope = rememberCoroutineScope()

    resultStore.consumeResult<ImportOverrideByTextNavResult> { vm.importText(it.text) }

    resultStore.consumeResult<ImportOverrideByFileNavResult> {
        openFileLauncher.launch("application/json")
    }

    // On successful clear of overrides, show snackbar
    resultStore.consumeResult<ResetServerIpOverrideConfirmationNavResult> { result ->
        scope.launch {
            snackbarHostState.showSnackbarImmediately(
                message =
                    if (result.clearSuccessful) {
                        resources.getString(R.string.overrides_cleared)
                    } else {
                        resources.getString(R.string.error_occurred)
                    },
                actionLabel = null,
            )
        }
    }

    ServerIpOverridesScreen(
        state = state,
        onBackClick = dropUnlessResumed { navigator.goBack() },
        onInfoClick = dropUnlessResumed { navigator.navigate(ServerIpOverrideInfoNavKey) },
        onImportClick =
            dropUnlessResumed { overridesActive ->
                navigator.navigate(ImportOverridesNavKey(overridesActive))
            },
        onResetOverridesClick =
            dropUnlessResumed { navigator.navigate(ResetServerIpOverrideConfirmationNavKey) },
        snackbarHostState = snackbarHostState,
        modifier =
            Modifier.sharedBounds(
                rememberSharedContentState(key = FeatureIndicator.SERVER_IP_OVERRIDE),
                animatedVisibilityScope = animatedVisibilityScope,
            ),
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerIpOverridesScreen(
    state: Lc<Boolean, ServerIpOverridesUiState>,
    onBackClick: () -> Unit,
    onInfoClick: () -> Unit,
    onImportClick: (Boolean) -> Unit,
    onResetOverridesClick: () -> Unit,
    modifier: Modifier = Modifier,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
) {
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.server_ip_override),
        modifier = modifier,
        navigationIcon = {
            if (state.isModal()) {
                NavigateCloseIconButton(onBackClick)
            } else {
                unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBackClick) }
            }
        },
        actions = {
            TopBarActions(
                overridesActive = state.contentOrNull()?.overridesActive,
                onInfoClick = onInfoClick,
                onResetOverridesClick = onResetOverridesClick,
            )
        },
    ) { modifier ->
        Column(
            modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMarginNew)
        ) {
            ServerIpOverridesListItem(active = state.contentOrNull()?.overridesActive)

            Spacer(modifier = Modifier.weight(1f))
            SnackbarHost(hostState = snackbarHostState) { MullvadSnackbar(snackbarData = it) }
            PrimaryButton(
                onClick = { onImportClick(state.contentOrNull()?.overridesActive == true) },
                text = stringResource(R.string.import_overrides_import),
                modifier =
                    Modifier.padding(bottom = Dimens.screenBottomMargin)
                        .testTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG),
            )
        }
    }
}

@Composable
private fun TopBarActions(
    overridesActive: Boolean?,
    onInfoClick: () -> Unit,
    onResetOverridesClick: () -> Unit,
) {
    var showMenu by remember { mutableStateOf(false) }
    InfoIconButton(
        onClick = onInfoClick,
        modifier = Modifier.testTag(SERVER_IP_OVERRIDE_INFO_TEST_TAG),
    )
    IconButton(
        onClick = { showMenu = !showMenu },
        modifier = Modifier.testTag(SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG),
    ) {
        Icon(imageVector = Icons.Rounded.MoreVert, contentDescription = null)
    }
    DropdownMenu(
        modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer),
        expanded = showMenu,
        onDismissRequest = { showMenu = false },
    ) {
        DropdownMenuItem(
            text = { Text(text = stringResource(R.string.server_ip_overrides_reset)) },
            onClick = {
                showMenu = false
                onResetOverridesClick()
            },
            enabled = overridesActive == true,
            colors =
                MenuDefaults.itemColors(
                    leadingIconColor = MaterialTheme.colorScheme.onPrimary,
                    disabledLeadingIconColor =
                        MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
                ),
            leadingIcon = { Icon(Icons.Rounded.Delete, contentDescription = null) },
            modifier = Modifier.testTag(SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG),
        )
    }
}

private fun Lc<Boolean, ServerIpOverridesUiState>.isModal(): Boolean =
    when (this) {
        is Lc.Loading -> this.value
        is Lc.Content -> this.value.isModal
    }

private fun SettingsPatchError?.toString(resources: Resources) =
    when (this) {
        SettingsPatchError.DeserializePatched ->
            resources.getString(R.string.patch_not_matching_specification)
        is SettingsPatchError.InvalidOrMissingValue ->
            resources.getString(R.string.settings_patch_error_invalid_or_missing_value, value)
        SettingsPatchError.ParsePatch ->
            resources.getString(R.string.settings_patch_error_unable_to_parse)
        is SettingsPatchError.UnknownOrProhibitedKey ->
            resources.getString(R.string.settings_patch_error_unknown_or_prohibited_key, value)
        SettingsPatchError.ApplyPatch ->
            resources.getString(R.string.settings_patch_error_failed_to_apply_patch)
        SettingsPatchError.RecursionLimit ->
            resources.getString(R.string.settings_patch_error_recursion_limit)
        null -> resources.getString(R.string.settings_patch_success)
    }

@Composable
private fun InfoIconButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
    iconTint: Color = MaterialTheme.colorScheme.onPrimary,
) {
    IconButton(modifier = modifier, onClick = onClick) {
        Icon(
            imageVector = Icons.Rounded.Info,
            contentDescription = contentDescription,
            tint = iconTint,
        )
    }
}
