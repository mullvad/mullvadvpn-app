package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.TextFields
import androidx.compose.material.icons.filled.UploadFile
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.SheetState
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ImportOverridesByTextDestination
import com.ramcosta.composedestinations.generated.destinations.ResetServerIpOverridesConfirmationDestination
import com.ramcosta.composedestinations.generated.destinations.ServerIpOverridesInfoDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.InfoIconButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.cell.ServerIpOverridesCell
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.preview.ServerIpOverridesUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_IMPORT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_INFO_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.SettingsPatchError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesUiState
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Loaded.Active|Loaded.Inactive|Loading")
@Composable
private fun PreviewServerIpOverridesScreen(
    @PreviewParameter(ServerIpOverridesUiStatePreviewParameterProvider::class)
    state: ServerIpOverridesUiState
) {
    AppTheme {
        ServerIpOverridesScreen(
            state = state,
            onBackClick = {},
            onInfoClick = {},
            onResetOverridesClick = {},
            onImportByFile = {},
            onImportByText = {},
            SnackbarHostState(),
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun ServerIpOverrides(
    navigator: DestinationsNavigator,
    importByTextResult: ResultRecipient<ImportOverridesByTextDestination, String>,
    clearOverridesResult: ResultRecipient<ResetServerIpOverridesConfirmationDestination, Boolean>,
) {
    val vm = koinViewModel<ServerIpOverridesViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }

    val context = LocalContext.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is ServerIpOverridesUiSideEffect.ImportResult ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = sideEffect.error.toString(context),
                        actionLabel = null,
                    )
                }
        }
    }

    importByTextResult.OnNavResultValue(vm::importText)

    // On successful clear of overrides, show snackbar
    val scope = rememberCoroutineScope()
    clearOverridesResult.OnNavResultValue { clearSuccessful ->
        scope.launch {
            snackbarHostState.showSnackbarImmediately(
                message =
                    if (clearSuccessful) {
                        context.getString(R.string.overrides_cleared)
                    } else {
                        context.getString(R.string.error_occurred)
                    },
                actionLabel = null,
            )
        }
    }

    val openFileLauncher =
        rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) {
            if (it != null) {
                vm.importFile(it)
            }
        }

    ServerIpOverridesScreen(
        state,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onInfoClick = dropUnlessResumed { navigator.navigate(ServerIpOverridesInfoDestination) },
        onResetOverridesClick =
            dropUnlessResumed { navigator.navigate(ResetServerIpOverridesConfirmationDestination) },
        onImportByFile = dropUnlessResumed { openFileLauncher.launch("application/json") },
        onImportByText = dropUnlessResumed { navigator.navigate(ImportOverridesByTextDestination) },
        snackbarHostState,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerIpOverridesScreen(
    state: ServerIpOverridesUiState,
    onBackClick: () -> Unit,
    onInfoClick: () -> Unit,
    onResetOverridesClick: () -> Unit,
    onImportByFile: () -> Unit,
    onImportByText: () -> Unit,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
) {

    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var showBottomSheet by remember { mutableStateOf(false) }

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.server_ip_overrides),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = {
            TopBarActions(
                overridesActive = state.overridesActive,
                onInfoClick = onInfoClick,
                onResetOverridesClick = onResetOverridesClick,
            )
        },
    ) { modifier ->
        if (showBottomSheet && state.overridesActive != null) {
            ImportOverridesByBottomSheet(
                sheetState,
                { showBottomSheet = it },
                state.overridesActive!!,
                onImportByFile,
                onImportByText,
            )
        }

        Column(modifier = modifier.animateContentSize()) {
            ServerIpOverridesCell(active = state.overridesActive)

            Spacer(modifier = Modifier.weight(1f))
            SnackbarHost(hostState = snackbarHostState) { MullvadSnackbar(snackbarData = it) }
            PrimaryButton(
                onClick = { showBottomSheet = true },
                text = stringResource(R.string.import_overrides_import),
                modifier =
                    Modifier.padding(horizontal = Dimens.sideMargin)
                        .padding(bottom = Dimens.screenVerticalMargin)
                        .testTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG),
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ImportOverridesByBottomSheet(
    sheetState: SheetState,
    showBottomSheet: (Boolean) -> Unit,
    overridesActive: Boolean,
    onImportByFile: () -> Unit,
    onImportByText: () -> Unit,
) {
    val scope = rememberCoroutineScope()
    val onCloseSheet = {
        scope
            .launch { sheetState.hide() }
            .invokeOnCompletion {
                if (!sheetState.isVisible) {
                    showBottomSheet(false)
                }
            }
    }
    val backgroundColor: Color = MaterialTheme.colorScheme.surfaceContainer
    val onBackgroundColor: Color = MaterialTheme.colorScheme.onSurface

    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { showBottomSheet(false) },
    ) {
        HeaderCell(
            text = stringResource(id = R.string.server_ip_overrides_import_by),
            background = backgroundColor,
        )
        HorizontalDivider(color = onBackgroundColor)
        IconCell(
            imageVector = Icons.Default.UploadFile,
            title = stringResource(id = R.string.server_ip_overrides_import_by_file),
            onClick = {
                onImportByFile()
                onCloseSheet()
            },
            background = backgroundColor,
            modifier = Modifier.testTag(SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG),
        )
        IconCell(
            imageVector = Icons.Default.TextFields,
            title = stringResource(id = R.string.server_ip_overrides_import_by_text),
            onClick = {
                onImportByText()
                onCloseSheet()
            },
            background = backgroundColor,
            modifier = Modifier.testTag(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG),
        )
        if (overridesActive) {
            HorizontalDivider(color = onBackgroundColor)
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    modifier = Modifier.padding(Dimens.mediumPadding),
                    imageVector = Icons.Default.Info,
                    tint = MaterialTheme.colorScheme.error,
                    contentDescription = null,
                )
                Text(
                    modifier =
                        Modifier.padding(
                            top = Dimens.smallPadding,
                            end = Dimens.mediumPadding,
                            bottom = Dimens.smallPadding,
                        ),
                    text = stringResource(R.string.import_overrides_bottom_sheet_override_warning),
                    maxLines = 2,
                    style = MaterialTheme.typography.bodySmall,
                    overflow = TextOverflow.Ellipsis,
                )
            }
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
        Icon(imageVector = Icons.Default.MoreVert, contentDescription = null)
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
            enabled = overridesActive ?: false,
            colors =
                MenuDefaults.itemColors(
                    leadingIconColor = MaterialTheme.colorScheme.onPrimary,
                    disabledLeadingIconColor =
                        MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
                ),
            leadingIcon = { Icon(Icons.Filled.Delete, contentDescription = null) },
            modifier = Modifier.testTag(SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG),
        )
    }
}

private fun SettingsPatchError?.toString(context: Context) =
    when (this) {
        SettingsPatchError.DeserializePatched ->
            context.getString(R.string.patch_not_matching_specification)
        is SettingsPatchError.InvalidOrMissingValue ->
            context.getString(R.string.settings_patch_error_invalid_or_missing_value, value)
        SettingsPatchError.ParsePatch ->
            context.getString(R.string.settings_patch_error_unable_to_parse)
        is SettingsPatchError.UnknownOrProhibitedKey ->
            context.getString(R.string.settings_patch_error_unknown_or_prohibited_key, value)
        SettingsPatchError.ApplyPatch ->
            context.getString(R.string.settings_patch_error_failed_to_apply_patch)
        SettingsPatchError.RecursionLimit ->
            context.getString(R.string.settings_patch_error_recursion_limit)
        null -> context.getString(R.string.settings_patch_success)
    }
