package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.edit

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.TextObfuscationMode
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SecureTextField
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.toMutableStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ExpandableListItem
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListHeader
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@Composable
fun CustomTunnelEdit(
    navigator: Navigator,
    isNew: Boolean,
) {
    val vm = koinViewModel<CustomTunnelEditViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is PersonalVpnSideEffect.FailedToSave ->
                snackbarHostState.showSnackbarImmediately(sideEffect.reason)

            PersonalVpnSideEffect.FormErrors ->
                snackbarHostState.showSnackbarImmediately(
                    "Could not save configuration, form contains errors"
                )

            PersonalVpnSideEffect.ConfigurationSaved ->
                snackbarHostState.showSnackbarImmediately("Configuration has been saved")
        }
    }

    CustomTunnelEditScreen(
        state = state,
        isNew = isNew,
        onBackClick = dropUnlessResumed { navigator.goBack() },
        onClearError = vm::onClearError,
        onClearAllowedIpErrors = vm::onClearAllowedIpErrors,
        onClearTunnelIpErrors = vm::onClearTunnelIpErrors,
        saveConfig = vm::save,
        clearConfig = vm::clearConfig,
        snackbarHostState = snackbarHostState,
    )
}

@Preview
@Composable
private fun PreviewCustomTunnelEditScreen() {
    AppTheme {
        CustomTunnelEditScreen(
            state = Lc.Content(PersonalVpnUiState(clearEnabled = false)),
            isNew = false,
            onBackClick = {},
            saveConfig = {},
            clearConfig = {},
            onClearTunnelIpErrors = {},
            onClearAllowedIpErrors = {},
            onClearError = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomTunnelEditScreen(
    state: Lc<Boolean, PersonalVpnUiState>,
    isNew: Boolean,
    onBackClick: () -> Unit,
    modifier: Modifier = Modifier,
    onClearError: (FormDataError) -> Unit,
    onClearAllowedIpErrors: () -> Unit,
    onClearTunnelIpErrors: () -> Unit,
    saveConfig: (PersonalVpnFormData) -> Unit,
    clearConfig: () -> Unit,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
) {
    if (state !is Lc.Content) return

    val initialFormData = state.value.initialFormData

    val nameFieldState = key(initialFormData) { rememberTextFieldState(initialFormData.name) }
    LaunchedEffect(nameFieldState.text) {
        val error = state.value.nameDataError ?: return@LaunchedEffect
        onClearError(error)
    }

    val privateKeyTextFieldState =
        key(initialFormData) { rememberTextFieldState(initialFormData.privateKey) }
    LaunchedEffect(privateKeyTextFieldState.text) {
        val error = state.value.privateKeyDataError ?: return@LaunchedEffect
        onClearError(error)
    }
    val tunnelIpTextFieldStates =
        remember(initialFormData) {
            initialFormData.tunnelIps.map { TextFieldState(it) }.toMutableStateList()
        }

    // Clear per-field tunnel IP error when text changes
    tunnelIpTextFieldStates.forEachIndexed { index, textFieldState ->
        LaunchedEffect(textFieldState.text) {
            val error = state.value.tunnelIpDataErrors[index] ?: return@LaunchedEffect
            onClearError(error)
        }
    }
    val publicKeyTextFieldState =
        key(initialFormData) { rememberTextFieldState(initialFormData.publicKey) }
    LaunchedEffect(publicKeyTextFieldState.text) {
        val error = state.value.publicKeyDataError ?: return@LaunchedEffect
        onClearError(error)
    }

    val allowedIpTextFieldStates =
        remember(initialFormData) {
            initialFormData.allowedIPs.map { TextFieldState(it) }.toMutableStateList()
        }

    // Clear per-field allowed IP error when text changes
    allowedIpTextFieldStates.forEachIndexed { index, textFieldState ->
        LaunchedEffect(textFieldState.text) {
            val error = state.value.allowedIpDataErrors[index] ?: return@LaunchedEffect
            onClearError(error)
        }
    }

    val endpointAddressTextFieldState =
        key(initialFormData) { rememberTextFieldState(initialFormData.endpointAddress) }
    LaunchedEffect(endpointAddressTextFieldState.text) {
        val error = state.value.endpointAddressDataError ?: return@LaunchedEffect
        onClearError(error)
    }

    val endpointPortTextFieldState =
        key(initialFormData) {
            rememberTextFieldState(initialFormData.endpointPort?.toString() ?: "")
        }
    LaunchedEffect(endpointPortTextFieldState.text) {
        val error = state.value.endpointPortDataError ?: return@LaunchedEffect
        onClearError(error)
    }

    val currentAllowedIps = allowedIpTextFieldStates.map { it.text.toString() }
    val currentTunnelIps = tunnelIpTextFieldStates.map { it.text.toString() }
    val formHasChanges =
        nameFieldState.text != initialFormData.name ||
            privateKeyTextFieldState.text != initialFormData.privateKey ||
            currentTunnelIps != initialFormData.tunnelIps ||
            publicKeyTextFieldState.text != initialFormData.publicKey ||
            currentAllowedIps != initialFormData.allowedIPs ||
            endpointAddressTextFieldState.text != initialFormData.endpointAddress ||
            endpointPortTextFieldState.text != initialFormData.endpointPort.toString()

    ScaffoldWithSmallTopBar(
        appBarTitle =
            if (isNew) {
                stringResource(id = R.string.create_configuration)
            } else {
                val name = initialFormData.name
                stringResource(id = R.string.edit_configuration, name)
            },
        modifier = modifier,
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = {
            Actions(
                isNew = isNew,
                onSavedEnabled = isNew || formHasChanges,
                onSaveClick = {
                    saveConfig(
                        PersonalVpnFormData(
                            privateKey = privateKeyTextFieldState.text.toString(),
                            tunnelIps = tunnelIpTextFieldStates.map { it.text.toString() },
                            publicKey = publicKeyTextFieldState.text.toString(),
                            allowedIPs = allowedIpTextFieldStates.map { it.text.toString() },
                            endpointAddress = endpointAddressTextFieldState.text.toString(),
                            endpointPort = endpointPortTextFieldState.text.toString().toIntOrNull(),
                        )
                    )
                },
                onDeleteClick = clearConfig,
            )
        },
        snackbarHostState = snackbarHostState,
    ) { modifier ->
        Column(
            modifier =
                modifier
                    .verticalScroll(rememberScrollState())
                    .animateContentSize()
                    .padding(horizontal = Dimens.sideMarginNew)
        ) {
            TextField(
                modifier = Modifier.fillMaxWidth().padding(bottom = Dimens.cellVerticalSpacing),
                state = nameFieldState,
                label = { Text(stringResource(R.string.name)) },
                isError = state.value.nameDataError != null,
                lineLimits = TextFieldLineLimits.SingleLine,
                colors = mullvadDarkTextFieldColors(),
                supportingText = state.value.nameDataError?.let { { Text(it.toErrorMessage()) } },
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
            )

            ListHeader(text = stringResource(R.string.custom_tunnels_edit_interface))

            var showPassword by remember { mutableStateOf(false) }
            SecureTextField(
                modifier = Modifier.fillMaxWidth(),
                state = privateKeyTextFieldState,
                label = { Text(stringResource(R.string.private_key)) },
                isError = state.value.privateKeyDataError != null,
                placeholder = { Text(KEY_PLACEHOLDER) },
                colors = mullvadDarkTextFieldColors(),
                textObfuscationMode =
                    if (showPassword) TextObfuscationMode.Visible
                    else TextObfuscationMode.RevealLastTyped,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                trailingIcon = {
                    IconButton(onClick = { showPassword = !showPassword }) {
                        Icon(
                            imageVector =
                                if (showPassword) Icons.Outlined.VisibilityOff
                                else Icons.Outlined.Visibility,
                            contentDescription =
                                if (showPassword) stringResource(id = R.string.hide_account_number)
                                else stringResource(id = R.string.show_account_number),
                        )
                    }
                },
                supportingText =
                    state.value.privateKeyDataError?.let { { Text(it.toErrorMessage()) } },
            )

            // Tunnel addresses - dynamic list of text fields
            tunnelIpTextFieldStates.forEachIndexed { index, textFieldState ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.Top,
                ) {
                    TextField(
                        modifier = Modifier.padding(Dimens.miniPadding).weight(1f),
                        state = textFieldState,
                        placeholder = { Text(IP_ADDRESS_PLACEHOLDER) },
                        label = { Text(stringResource(R.string.address)) },
                        isError = state.value.tunnelIpDataErrors.containsKey(index),
                        lineLimits = TextFieldLineLimits.SingleLine,
                        colors = mullvadDarkTextFieldColors(),
                        supportingText =
                            state.value.tunnelIpDataErrors[index]?.let {
                                { Text(it.toErrorMessage()) }
                            },
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                        trailingIcon = {
                            IconButton(
                                onClick = {
                                    tunnelIpTextFieldStates.removeAt(index)
                                    onClearTunnelIpErrors()
                                }
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = stringResource(R.string.delete),
                                )
                            }
                        },
                    )
                }
            }
            TextButton(
                onClick = { tunnelIpTextFieldStates.add(TextFieldState("")) },
                colors =
                    ButtonDefaults.buttonColors(contentColor = MaterialTheme.colorScheme.onSurface),
            ) {
                Icon(imageVector = Icons.Outlined.Add, contentDescription = null)
                Text(stringResource(R.string.add_address))
            }

            var expanded by remember { mutableStateOf(true) }
            ExpandableListItem(
                modifier = Modifier.fillMaxWidth(),
                title = stringResource(R.string.peer),
                isExpanded = expanded,
                onCellClicked = { expanded = !expanded },
            )
            if (expanded) {
                Column {
                    TextField(
                        modifier = Modifier.fillMaxWidth(),
                        state = publicKeyTextFieldState,
                        label = { Text(stringResource(R.string.public_key)) },
                        placeholder = { Text(KEY_PLACEHOLDER) },
                        isError = state.value.publicKeyDataError != null,
                        colors = mullvadDarkTextFieldColors(),
                        lineLimits = TextFieldLineLimits.SingleLine,
                        supportingText =
                            state.value.publicKeyDataError?.let { { Text(it.toErrorMessage()) } },
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    )

                    // Allowed IPs - dynamic list of text fields
                    allowedIpTextFieldStates.forEachIndexed { index, textFieldState ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.Top,
                        ) {
                            TextField(
                                modifier = Modifier.padding(Dimens.miniPadding).weight(1f),
                                state = textFieldState,
                                label = { Text(stringResource(R.string.allowed_ips)) },
                                placeholder = { Text(IP_ADDRESS_PLACEHOLDER) },
                                isError = state.value.allowedIpDataErrors.containsKey(index),
                                lineLimits = TextFieldLineLimits.SingleLine,
                                colors = mullvadDarkTextFieldColors(),
                                supportingText =
                                    state.value.allowedIpDataErrors[index]?.let {
                                        { Text(it.toErrorMessage()) }
                                    },
                                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                                trailingIcon = {
                                    if (allowedIpTextFieldStates.size > 1) {
                                        IconButton(
                                            onClick = {
                                                allowedIpTextFieldStates.removeAt(index)
                                                onClearAllowedIpErrors()
                                            }
                                        ) {
                                            Icon(
                                                imageVector = Icons.Default.Delete,
                                                contentDescription =
                                                    stringResource(R.string.delete),
                                            )
                                        }
                                    }
                                },
                            )
                        }
                    }
                    TextButton(
                        onClick = { allowedIpTextFieldStates.add(TextFieldState("")) },
                        colors =
                            ButtonDefaults.buttonColors(
                                contentColor = MaterialTheme.colorScheme.onSurface
                            ),
                    ) {
                        Icon(
                            imageVector = Icons.Outlined.Add,
                            contentDescription = null,
                        )
                        Text(stringResource(R.string.add_ip))
                    }

                    TextField(
                        modifier = Modifier.fillMaxWidth(),
                        state = endpointAddressTextFieldState,
                        label = { Text(stringResource(R.string.endpoint)) },
                        placeholder = { Text(ENDPOINT_PLACEHOLDER) },
                        isError = state.value.endpointAddressDataError != null,
                        colors = mullvadDarkTextFieldColors(),
                        supportingText =
                            state.value.endpointAddressDataError?.let {
                                { Text(it.toErrorMessage()) }
                            },
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                        lineLimits = TextFieldLineLimits.SingleLine,
                    )

                    TextField(
                        modifier = Modifier.fillMaxWidth(),
                        state = endpointPortTextFieldState,
                        label = { Text(stringResource(R.string.port)) },
                        placeholder = { Text(PORT_PLACEHOLDER) },
                        isError = state.value.endpointPortDataError != null,
                        colors = mullvadDarkTextFieldColors(),
                        supportingText =
                            state.value.endpointPortDataError?.let {
                                { Text(it.toErrorMessage()) }
                            },
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                        lineLimits = TextFieldLineLimits.SingleLine,
                    )
                }
            }
        }
    }
}

@Composable
private fun Actions(
    isNew: Boolean,
    onSavedEnabled: Boolean,
    onSaveClick: () -> Unit,
    onDeleteClick: () -> Unit,
) {
    if (!isNew) {
        IconButton(onClick = onDeleteClick) {
            Icon(
                imageVector = Icons.Default.Delete,
                contentDescription = stringResource(R.string.delete),
            )
        }
    }
    TextButton(
        onClick = onSaveClick,
        enabled = onSavedEnabled,
        colors =
            ButtonDefaults.textButtonColors()
                .copy(contentColor = MaterialTheme.colorScheme.onPrimary),
        modifier = Modifier.testTag(SAVE_BUTTON_TEST_TAG),
    ) {
        Text(text = stringResource(R.string.save), style = MaterialTheme.typography.labelLarge)
    }
}

@Composable
private fun FormDataError.toErrorMessage(): String =
    when (this) {
        is FormDataError.AllowedIp -> "Bad allowed IP"
        is FormDataError.EndpointAddress -> this.toString()
        is FormDataError.EndpointPort -> this.toString()
        is FormDataError.PrivateKey -> keyParseError.toString()
        is FormDataError.PublicKey -> keyParseError.toString()
        is FormDataError.TunnelIp -> "Bad address IP"
        is FormDataError.Name -> stringResource(R.string.this_field_is_required)
    }

private const val KEY_PLACEHOLDER = "AAAA...AAAA="
private const val IP_ADDRESS_PLACEHOLDER = "10.0.0.2"
private const val ENDPOINT_PLACEHOLDER = "10.1.2.3.4 or example.com"
private const val PORT_PLACEHOLDER = "12345"
