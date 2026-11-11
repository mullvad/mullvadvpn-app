package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.TextObfuscationMode
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SecureTextField
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.runtime.toMutableStateList
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import java.util.UUID
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customvpn.api.RemoveCustomTunnelConfigConfirmationNavKey
import net.mullvad.mullvadvpn.feature.customvpn.api.RemoveCustomTunnelConfigConfirmationNavResult
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.util.then
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.ui.component.AnimatedVisibilityItem
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.ListItemIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ExpandableListItem
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListHeader
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryTextButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewCustomTunnelEditScreen() {
    AppTheme {
        CustomTunnelEditScreen(
            state = Lc.Content(CustomTunnelEditUiState()),
            isNew = false,
            onBackClick = {},
            saveConfig = {},
            clearConfig = {},
            snackbarHostState = remember { SnackbarHostState() },
        )
    }
}

class PeerState(
    val id: String = UUID.randomUUID().toString(),
    val publicKey: TextFieldState = TextFieldState(),
    val allowedIps: SnapshotStateList<FormTextField> = mutableStateListOf(FormTextField()),
    val endpointAddress: TextFieldState = TextFieldState(),
    val endpointPort: TextFieldState = TextFieldState(),
    var isNewlyAdded: Boolean = false,
)

class FormTextField(
    val id: String = UUID.randomUUID().toString(),
    val state: TextFieldState = TextFieldState(),
    var isNewlyAdded: Boolean = false,
)

@Composable
fun CustomTunnelEdit(
    navigator: Navigator,
    isNewConfig: Boolean,
) {
    val vm = koinViewModel<CustomTunnelEditViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }
    var isNew by rememberSaveable { mutableStateOf(isNewConfig) }

    LocalResultStore.current.consumeResult<RemoveCustomTunnelConfigConfirmationNavResult> {
        isNew = true
        vm.clearConfig()
    }

    val formHasErrorMessage = stringResource(R.string.custom_tunnels_form_contains_errors_message)
    val saveSuccessMessage = stringResource(R.string.custom_tunnels_form_saved_success)

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is PersonalVpnSideEffect.FailedToSave ->
                snackbarHostState.showSnackbarImmediately(sideEffect.reason)

            PersonalVpnSideEffect.FormErrors ->
                snackbarHostState.showSnackbarImmediately(formHasErrorMessage)

            PersonalVpnSideEffect.ConfigurationSaved ->
                snackbarHostState.showSnackbarImmediately(saveSuccessMessage)
        }
    }

    CustomTunnelEditScreen(
        state = state,
        isNew = isNew,
        onBackClick = dropUnlessResumed { navigator.goBack() },
        saveConfig = vm::save,
        clearConfig =
            dropUnlessResumed<PersonalVpnFormData> {
                navigator.navigate(RemoveCustomTunnelConfigConfirmationNavKey(name = it.name))
            },
        snackbarHostState = snackbarHostState,
    )
}

@Suppress("LongMethod")
@Composable
fun CustomTunnelEditScreen(
    state: Lc<Boolean, CustomTunnelEditUiState>,
    isNew: Boolean,
    onBackClick: () -> Unit,
    modifier: Modifier = Modifier,
    saveConfig: (PersonalVpnFormData) -> Unit,
    clearConfig: (PersonalVpnFormData) -> Unit,
    snackbarHostState: SnackbarHostState,
) {
    if (state !is Lc.Content) {
        MullvadCircularProgressIndicatorLarge()
        return
    }

    val formData = state.value.formData

    val nameFieldState = key(formData) { rememberTextFieldState(formData.name) }

    val privateKeyTextFieldState = key(formData) { rememberTextFieldState(formData.privateKey) }

    val tunnelIpTextFieldStates =
        rememberSaveable(formData, saver = FormTextFieldListSaver) {
            formData.tunnelIps
                .map { address -> FormTextField(state = TextFieldState(address)) }
                .toMutableStateList()
        }

    val peerStates =
        rememberSaveable(formData, saver = PeerStateListSaver) {
            formData.peers
                .map { peer ->
                    PeerState(
                        id = peer.id,
                        publicKey = TextFieldState(peer.publicKey),
                        allowedIps =
                            peer.allowedIps
                                .ifEmpty { listOf("") }
                                .map { address -> FormTextField(state = TextFieldState(address)) }
                                .toMutableStateList(),
                        endpointAddress = TextFieldState(peer.endpointAddress),
                        endpointPort = TextFieldState(peer.endpointPort),
                    )
                }
                .toMutableStateList()
        }

    val currentFormData =
        PersonalVpnFormData(
            name = nameFieldState.text.toString(),
            enabled = formData.enabled,
            privateKey = privateKeyTextFieldState.text.toString(),
            tunnelIps = tunnelIpTextFieldStates.map { it.state.text.toString() }.toSet(),
            peers =
                peerStates.map { peer ->
                    PeerFormData(
                        id = peer.id,
                        publicKey = peer.publicKey.text.toString(),
                        allowedIps = peer.allowedIps.map { it.state.text.toString() }.toSet(),
                        endpointAddress = peer.endpointAddress.text.toString(),
                        endpointPort = peer.endpointPort.text.toString(),
                    )
                },
        )

    val formHasChanges = currentFormData != formData

    ScaffoldWithSmallTopBar(
        appBarTitle =
            if (isNew) {
                stringResource(id = R.string.create_configuration)
            } else {
                stringResource(id = R.string.edit_configuration, formData.name)
            },
        modifier = modifier,
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = {
            Actions(
                isNew = isNew,
                isSavedEnabled = isNew || formHasChanges,
                onSaveClick = { saveConfig(currentFormData) },
                onDeleteClick = { clearConfig(currentFormData) },
            )
        },
        snackbarHostState = snackbarHostState,
    ) { contentModifier ->
        LazyColumn(
            modifier = contentModifier.fillMaxSize().padding(horizontal = Dimens.sideMarginNew)
        ) {
            interfaceSection(
                nameFieldState = nameFieldState,
                nameError = state.value.formErrors?.nameError,
                privateKeyTextFieldState = privateKeyTextFieldState,
                privateKeyError = state.value.formErrors?.privateKeyError,
                tunnelIpTextFieldStates = tunnelIpTextFieldStates,
                tunnelIpErrors = state.value.formErrors?.tunnelIpErrors.orEmpty(),
            )

            item { Spacer(Modifier.height(Dimens.verticalSpace)) }

            peersSection(
                peerStates = peerStates,
                state = state.value,
            )

            item { Spacer(Modifier.height(Dimens.screenBottomMarginNew)) }
        }
    }
}

@Suppress("LongParameterList")
fun LazyListScope.interfaceSection(
    modifier: Modifier = Modifier,
    nameFieldState: TextFieldState,
    nameError: FormDataError?,
    privateKeyTextFieldState: TextFieldState,
    privateKeyError: FormDataError.PrivateKey?,
    tunnelIpTextFieldStates: SnapshotStateList<FormTextField>,
    tunnelIpErrors: Map<String, FormDataError>,
) {
    item {
        Column(modifier = modifier) {
            TextField(
                modifier = Modifier.fillMaxWidth(),
                state = nameFieldState,
                label = { Text(stringResource(R.string.name)) },
                isError = nameError != null,
                lineLimits = TextFieldLineLimits.SingleLine,
                colors = mullvadDarkTextFieldColors(),
                supportingText = nameError?.let { { Text(it.toErrorMessage()) } },
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
            )

            Spacer(Modifier.height(Dimens.verticalSpace))
            ListHeader(text = stringResource(R.string.custom_tunnels_edit_interface))

            val isErr = privateKeyError?.errorValue == privateKeyTextFieldState.text
            var showPassword by rememberSaveable { mutableStateOf(false) }

            SecureTextField(
                modifier = Modifier.fillMaxWidth(),
                state = privateKeyTextFieldState,
                label = { Text(stringResource(R.string.private_key)) },
                isError = privateKeyError?.errorValue == privateKeyTextFieldState.text,
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
                supportingText = privateKeyError?.let { isErr.then { Text(it.toErrorMessage()) } },
            )
        }
    }

    this.tunnelIpFieldList(
        tunnelIpFormFieldStates = tunnelIpTextFieldStates,
        tunnelIpErrors = tunnelIpErrors,
    )

    item {
        PrimaryTextButton(
            modifier = Modifier.padding(ButtonVerticalPadding),
            onClick = { tunnelIpTextFieldStates.add(FormTextField(isNewlyAdded = true)) },
            colors =
                ButtonDefaults.textButtonColors()
                    .copy(contentColor = MaterialTheme.colorScheme.onPrimary),
            leadingIcon = { Icon(imageVector = Icons.Outlined.Add, contentDescription = null) },
            text = stringResource(R.string.add_address),
            textStyle = MaterialTheme.typography.labelLarge,
            textDecoration = TextDecoration.Underline,
        )
    }
}

fun LazyListScope.tunnelIpFieldList(
    tunnelIpFormFieldStates: SnapshotStateList<FormTextField>,
    tunnelIpErrors: Map<String, FormDataError>,
) {
    itemsIndexed(
        items = tunnelIpFormFieldStates,
        key = { _, formFieldState -> formFieldState.id },
    ) { index, formFieldState ->
        AnimatedVisibilityItem(
            onRemoveConfirmed = { tunnelIpFormFieldStates.remove(formFieldState) },
            showEntryAnimation =
                formFieldState.isNewlyAdded.also { formFieldState.isNewlyAdded = false },
        ) { onRequestRemove ->
            Column {
                Spacer(Modifier.height(Dimens.verticalSpace))

                val err = tunnelIpErrors[formFieldState.state.text]

                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = formFieldState.state,
                    placeholder = { Text(IP_ADDRESS_PLACEHOLDER) },
                    label = { Text(stringResource(R.string.address)) },
                    isError = err != null,
                    lineLimits = TextFieldLineLimits.SingleLine,
                    colors = mullvadDarkTextFieldColors(),
                    supportingText = err?.let { { Text(it.toErrorMessage()) } },
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    trailingIcon =
                        (index > 0).then {
                            IconButton(onClick = onRequestRemove) {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = stringResource(R.string.delete),
                                )
                            }
                        },
                )
            }
        }
    }
}

fun LazyListScope.peersSection(
    peerStates: SnapshotStateList<PeerState>,
    state: CustomTunnelEditUiState,
) {
    item { ListHeader(text = stringResource(R.string.custom_tunnels_edit_peers)) }

    itemsIndexed(
        items = peerStates,
        key = { _, peerState -> peerState.id },
    ) { index, peerState ->
        AnimatedVisibilityItem(
            onRemoveConfirmed = { peerStates.remove(peerState) },
            showEntryAnimation =
                if (peerState.isNewlyAdded) {
                    peerState.isNewlyAdded = false
                    true
                } else {
                    false
                },
        ) { onRequestRemove ->
            Column {
                val peerTitle =
                    pluralStringResource(R.plurals.custom_tunnel_peer_names, index + 1, index + 1)

                PeerItem(
                    title = peerTitle,
                    peerState = peerState,
                    canDelete = index > 0,
                    onDeletePeer = onRequestRemove,
                    publicKeyError = state.formErrors?.peerErrors[peerState.id]?.publicKeyError,
                    endpointAddressError =
                        state.formErrors?.peerErrors[peerState.id]?.endpointAddressError,
                    endpointPortError =
                        state.formErrors?.peerErrors[peerState.id]?.endpointPortError,
                    allowedIpErrors =
                        state.formErrors?.peerErrors[peerState.id]?.allowedIpErrors.orEmpty(),
                )
                Spacer(Modifier.height(Dimens.verticalSpace))
            }
        }
    }

    item {
        PrimaryTextButton(
            onClick = {
                peerStates.add(
                    PeerState(
                        publicKey = TextFieldState(""),
                        allowedIps = mutableStateListOf(FormTextField()),
                        endpointAddress = TextFieldState(""),
                        endpointPort = TextFieldState(""),
                        isNewlyAdded = true,
                    )
                )
            },
            colors =
                ButtonDefaults.textButtonColors()
                    .copy(contentColor = MaterialTheme.colorScheme.onPrimary),
            leadingIcon = { Icon(imageVector = Icons.Outlined.Add, contentDescription = null) },
            text = stringResource(R.string.add_peer),
            textStyle = MaterialTheme.typography.labelLarge,
            textDecoration = TextDecoration.Underline,
        )
    }
}

@Suppress("LongMethod")
@Composable
fun PeerItem(
    modifier: Modifier = Modifier,
    title: String,
    peerState: PeerState,
    canDelete: Boolean,
    onDeletePeer: () -> Unit,
    publicKeyError: FormDataError.PublicKey?,
    endpointAddressError: FormDataError.EndpointAddress?,
    endpointPortError: FormDataError.EndpointPort?,
    allowedIpErrors: Map<String, FormDataError>,
) {
    var expanded by rememberSaveable { mutableStateOf(true) }

    ExpandableListItem(
        modifier = modifier.fillMaxWidth(),
        title = title,
        isExpanded = expanded,
        onCellClicked = { expanded = !expanded },
        extraTrailingContent =
            canDelete.then {
                ListItemIconButton(onClick = onDeletePeer) {
                    Icon(
                        imageVector = Icons.Rounded.Delete,
                        contentDescription = stringResource(R.string.delete),
                    )
                }
            },
        expandedContent = {
            Column(
                modifier =
                    Modifier.background(MaterialTheme.colorScheme.surfaceContainerLow)
                        .padding(horizontal = Dimens.sideMarginNew)
            ) {
                Spacer(Modifier.height(Dimens.verticalSpace))

                val keyError = publicKeyError?.errorValue == peerState.publicKey.text
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = peerState.publicKey,
                    label = { Text(stringResource(R.string.public_key)) },
                    placeholder = { Text(KEY_PLACEHOLDER) },
                    isError = keyError,
                    colors = mullvadDarkTextFieldColors(),
                    lineLimits = TextFieldLineLimits.SingleLine,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    supportingText =
                        publicKeyError?.let { keyError.then { Text(it.toErrorMessage()) } },
                )

                AllowedIpFieldList(
                    allowedIpTextFieldStates = peerState.allowedIps,
                    allowedIpErrors = allowedIpErrors,
                )

                PrimaryTextButton(
                    modifier = Modifier.padding(ButtonVerticalPadding),
                    onClick = { peerState.allowedIps.add(FormTextField(isNewlyAdded = true)) },
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
                        ),
                    leadingIcon = {
                        Icon(imageVector = Icons.Outlined.Add, contentDescription = null)
                    },
                    text = stringResource(R.string.add_ip),
                    textStyle = MaterialTheme.typography.labelLarge,
                    textDecoration = TextDecoration.Underline,
                )

                Spacer(Modifier.height(Dimens.verticalSpace))

                val addressError =
                    endpointAddressError?.errorValue == peerState.endpointAddress.text
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = peerState.endpointAddress,
                    label = { Text(stringResource(R.string.endpoint)) },
                    placeholder = {
                        Text(
                            stringResource(
                                R.string.x_or_y,
                                ENDPOINT_PLACEHOLDER_IPV4,
                                ENDPOINT_PLACEHOLDER_IPV6,
                            )
                        )
                    },
                    isError = addressError,
                    colors = mullvadDarkTextFieldColors(),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    lineLimits = TextFieldLineLimits.SingleLine,
                    supportingText =
                        endpointAddressError?.let {
                            addressError.then { Text(it.toErrorMessage()) }
                        },
                )

                Spacer(Modifier.height(Dimens.verticalSpace))

                val portError = endpointPortError?.errorValue == peerState.endpointPort.text
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = peerState.endpointPort,
                    label = { Text(stringResource(R.string.port)) },
                    placeholder = { Text(PORT_PLACEHOLDER) },
                    isError = portError,
                    colors = mullvadDarkTextFieldColors(),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    lineLimits = TextFieldLineLimits.SingleLine,
                    supportingText =
                        endpointPortError?.let { portError.then { Text(it.toErrorMessage()) } },
                )

                Spacer(Modifier.height(Dimens.verticalSpace))
            }
        },
    )
}

@Composable
fun AllowedIpFieldList(
    allowedIpTextFieldStates: SnapshotStateList<FormTextField>,
    allowedIpErrors: Map<String, FormDataError>,
) {
    Column {
        allowedIpTextFieldStates.forEachIndexed { index, textFieldState ->
            key(textFieldState.id) {
                AnimatedVisibilityItem(
                    onRemoveConfirmed = { allowedIpTextFieldStates.remove(textFieldState) },
                    showEntryAnimation =
                        if (textFieldState.isNewlyAdded) {
                            textFieldState.isNewlyAdded = false
                            true
                        } else {
                            false
                        },
                ) { onRequestRemove ->
                    Column {
                        Spacer(Modifier.height(Dimens.verticalSpace))

                        val error = allowedIpErrors[textFieldState.state.text]
                        TextField(
                            modifier = Modifier.fillMaxWidth(),
                            state = textFieldState.state,
                            label = { Text(stringResource(R.string.allowed_ips)) },
                            placeholder = { Text(IP_ADDRESS_PLACEHOLDER) },
                            isError = error != null,
                            lineLimits = TextFieldLineLimits.SingleLine,
                            colors = mullvadDarkTextFieldColors(),
                            supportingText = error?.let { { Text(it.toErrorMessage()) } },
                            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                            trailingIcon = {
                                if (index > 0) {
                                    IconButton(onClick = onRequestRemove) {
                                        Icon(
                                            imageVector = Icons.Default.Delete,
                                            contentDescription = stringResource(R.string.delete),
                                        )
                                    }
                                }
                            },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun Actions(
    isNew: Boolean,
    isSavedEnabled: Boolean,
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
        enabled = isSavedEnabled,
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
        is FormDataError.TunnelIp,
        is FormDataError.AllowedIp,
        is FormDataError.EndpointAddress -> stringResource(R.string.invalid_ip_address)
        is FormDataError.PrivateKey -> stringResource(R.string.invalid_private_key)
        is FormDataError.PublicKey -> stringResource(R.string.invalid_public_key)
        is FormDataError.EmptyName -> stringResource(R.string.custom_tunnels_name_cannot_be_empty)
        is FormDataError.EndpointPort -> stringResource(R.string.invalid_port, 1, Port.MAX_VALUE)
    }

private const val KEY_PLACEHOLDER = "AAAA...AAAA="
private const val IP_ADDRESS_PLACEHOLDER = "10.0.0.2"
private const val ENDPOINT_PLACEHOLDER_IPV4 = "198.51.100.5"
private const val ENDPOINT_PLACEHOLDER_IPV6 = "2001:DB8::1"
private const val PORT_PLACEHOLDER = "12345"

private val ButtonVerticalPadding = 8.dp
