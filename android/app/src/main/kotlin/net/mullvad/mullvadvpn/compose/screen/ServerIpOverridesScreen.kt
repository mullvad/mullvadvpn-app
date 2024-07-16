package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ImportOverridesByTextDestination
import com.ramcosta.composedestinations.generated.destinations.ImportOverridesSheetDestination
import com.ramcosta.composedestinations.generated.destinations.ResetServerIpOverridesConfirmationDestination
import com.ramcosta.composedestinations.generated.destinations.ServerIpOverridesInfoDialogDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.InfoIconButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.ServerIpOverridesCell
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_IMPORT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_INFO_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightLeafTransition
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.SettingsPatchError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesViewModel
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesViewState
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewServerIpOverridesScreen() {
    AppTheme {
        ServerIpOverridesScreen(
            ServerIpOverridesViewState.Loaded(false),
            onBackClick = {},
            onInfoClick = {},
            onResetOverridesClick = {},
            showBottomSheet = {},
            SnackbarHostState())
    }
}

@Destination<RootGraph>(style = SlideInFromRightLeafTransition::class)
@Composable
fun ServerIpOverrides(
    navigator: DestinationsNavigator,
    importByTextResult: ResultRecipient<ImportOverridesByTextDestination, String>,
    importByFileResult: ResultRecipient<ImportOverridesSheetDestination, Boolean>,
    clearOverridesResult: ResultRecipient<ResetServerIpOverridesConfirmationDestination, Boolean>,
) {
    val vm = koinViewModel<ServerIpOverridesViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }

    val openFileLauncher =
        rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) {
            if (it != null) {
                vm.importFile(it)
            }
        }
    val context = LocalContext.current
    LaunchedEffectCollect(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is ServerIpOverridesUiSideEffect.ImportResult ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = sideEffect.error.toString(context), actionLabel = null)
                }
        }
    }

    importByFileResult.OnNavResultValue { openFileLauncher.launch("application/json") }
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
                actionLabel = null)
        }
    }

    ServerIpOverridesScreen(
        state,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onInfoClick =
            dropUnlessResumed { navigator.navigate(ServerIpOverridesInfoDialogDestination) },
        onResetOverridesClick =
            dropUnlessResumed { navigator.navigate(ResetServerIpOverridesConfirmationDestination) },
        showBottomSheet = dropUnlessResumed { navigator.navigate(ImportOverridesSheetDestination) },
        snackbarHostState)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerIpOverridesScreen(
    state: ServerIpOverridesViewState,
    onBackClick: () -> Unit,
    onInfoClick: () -> Unit,
    onResetOverridesClick: () -> Unit,
    showBottomSheet: () -> Unit,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() }
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.server_ip_overrides),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
        actions = {
            TopBarActions(
                overridesActive = state.overridesActive,
                onInfoClick = onInfoClick,
                onResetOverridesClick = onResetOverridesClick)
        }) { modifier ->
            Column(
                modifier = modifier.animateContentSize(),
            ) {
                ServerIpOverridesCell(active = state.overridesActive)

                Spacer(modifier = Modifier.weight(1f))
                PrimaryButton(
                    onClick = showBottomSheet,
                    text = stringResource(R.string.server_ip_overrides_import_button),
                    modifier =
                        Modifier.padding(horizontal = Dimens.sideMargin)
                            .padding(bottom = Dimens.screenVerticalMargin)
                            .testTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG),
                )
                SnackbarHost(
                    hostState = snackbarHostState, modifier = Modifier.animateContentSize()) {
                        MullvadSnackbar(snackbarData = it)
                    }
            }
        }
}

@Composable
private fun TopBarActions(
    overridesActive: Boolean?,
    onInfoClick: () -> Unit,
    onResetOverridesClick: () -> Unit
) {
    var showMenu by remember { mutableStateOf(false) }
    InfoIconButton(
        onClick = onInfoClick, modifier = Modifier.testTag(SERVER_IP_OVERRIDE_INFO_TEST_TAG))
    IconButton(
        onClick = { showMenu = !showMenu },
        modifier = Modifier.testTag(SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG)) {
            Icon(painterResource(id = R.drawable.icon_more_vert), contentDescription = null)
        }
    DropdownMenu(
        modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer),
        expanded = showMenu,
        onDismissRequest = { showMenu = false }) {
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
                            MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled)),
                leadingIcon = {
                    Icon(
                        Icons.Filled.Delete,
                        contentDescription = null,
                    )
                },
                modifier = Modifier.testTag(SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG))
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
