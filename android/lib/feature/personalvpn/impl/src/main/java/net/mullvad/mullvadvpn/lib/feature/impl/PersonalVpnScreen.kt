package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.TextObfuscationMode
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SecureTextField
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldLabelPosition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.toMutableStateList
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import java.time.Instant
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.Shapes
import org.koin.androidx.compose.koinViewModel

@Composable
fun SharedTransitionScope.PersonalVpn(
    navigator: Navigator,
    isModal: Boolean,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val vm = koinViewModel<PersonalVpnViewModel>()
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

    PersonalVpnScreen(
        state = state,
        isModal = isModal,
        onBackClick = dropUnlessResumed { navigator.goBack() },
        vm::onToggle,
        vm::onClearError,
        onClearAllowedIpErrors = vm::onClearAllowedIpErrors,
        onClearTunnelIpErrors = vm::onClearTunnelIpErrors,
        vm::save,
        clearConfig = vm::clearConfig,
        importConfig = vm::import,
        modifier =
            Modifier.sharedBounds(
                rememberSharedContentState(key = FeatureIndicator.PERSONAL_VPN),
                animatedVisibilityScope = animatedVisibilityScope,
            ),
        snackbarHostState = snackbarHostState,
    )
}

@Preview
@Composable
private fun PreviewPersonalVpnScreen() {
    AppTheme {
        PersonalVpnScreen(
            state =
                Lc.Content(
                    PersonalVpnUiState(
                        enabled = true,
                        clearEnabled = false,
                        tunnelStats = TunnelStats(),
                    )
                ),
            isModal = false,
            onBackClick = {},
            onTogglePersonalVpn = {},
            saveConfig = {},
            clearConfig = {},
            importConfig = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PersonalVpnScreen(
    state: Lc<Boolean, PersonalVpnUiState>,
    isModal: Boolean,
    onBackClick: () -> Unit,
    onTogglePersonalVpn: (Boolean) -> Unit,
    onClearError: (FormDataError) -> Unit = {},
    onClearAllowedIpErrors: () -> Unit = {},
    onClearTunnelIpErrors: () -> Unit = {},
    saveConfig: (PersonalVpnFormData) -> Unit,
    clearConfig: () -> Unit,
    importConfig: (String) -> Unit,
    modifier: Modifier = Modifier,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
) {
    if (state !is Lc.Content) return

    val initialFormData = state.value.initialFormData
    val privateKeyTextFieldState =
        key(initialFormData) { rememberTextFieldState(initialFormData.privateKey) }
    LaunchedEffect(privateKeyTextFieldState.text) {
        val error = state.value.privateKeyDataError ?: return@LaunchedEffect
        onClearError(error)
    }
    val tunnelIpTextFieldStates = remember(initialFormData) {
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

    val allowedIpTextFieldStates = remember(initialFormData) {
        initialFormData.allowedIPs.map { TextFieldState(it) }.toMutableStateList()
    }

    // Clear per-field allowed IP error when text changes
    allowedIpTextFieldStates.forEachIndexed { index, textFieldState ->
        LaunchedEffect(textFieldState.text) {
            val error = state.value.allowedIpDataErrors[index] ?: return@LaunchedEffect
            onClearError(error)
        }
    }

    val endpointTextFieldState =
        key(initialFormData) { rememberTextFieldState(initialFormData.endpoint) }
    LaunchedEffect(endpointTextFieldState.text) {
        val error = state.value.endpointDataError ?: return@LaunchedEffect
        onClearError(error)
    }

    val currentAllowedIps = allowedIpTextFieldStates.map { it.text.toString() }
    val currentTunnelIps = tunnelIpTextFieldStates.map { it.text.toString() }
    val formHasChanges =
        privateKeyTextFieldState.text != initialFormData.privateKey ||
            currentTunnelIps != initialFormData.tunnelIps ||
            publicKeyTextFieldState.text != initialFormData.publicKey ||
            currentAllowedIps != initialFormData.allowedIPs ||
            endpointTextFieldState.text != initialFormData.endpoint

    val context = LocalContext.current
    val openFileLauncher =
        rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
            if (uri != null) {
                val body =
                    context.contentResolver.openInputStream(uri)?.use { stream ->
                        stream.reader(Charsets.UTF_8).readText()
                    }
                if (body != null) importConfig(body)
            }
        }

    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.personal_vpn),
        modifier = modifier.imePadding(),
        navigationIcon = {
            if (isModal) {
                NavigateCloseIconButton(onBackClick)
            } else {
                NavigateBackIconButton(onNavigateBack = onBackClick)
            }
        },
        snackbarHostState = snackbarHostState,
        bottomBar = {
            Column(
                Modifier.windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom))
                    .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenBottomMargin),
                verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing),
            ) {
                PrimaryButton(
                    text = stringResource(R.string.save),
                    isEnabled = formHasChanges,
                    onClick = {
                        saveConfig(
                            PersonalVpnFormData(
                                privateKey = privateKeyTextFieldState.text.toString(),
                                tunnelIps =
                                    tunnelIpTextFieldStates.map { it.text.toString() },
                                publicKey = publicKeyTextFieldState.text.toString(),
                                allowedIPs =
                                    allowedIpTextFieldStates.map { it.text.toString() },
                                endpoint = endpointTextFieldState.text.toString(),
                            )
                        )
                    },
                )

                PrimaryButton(
                    text = "Import from file",
                    onClick = { openFileLauncher.launch(arrayOf("*/*")) },
                )

                NegativeButton(
                    text = "Delete",
                    isEnabled = state.value.clearEnabled,
                    onClick = {
                        privateKeyTextFieldState.clearText()
                        tunnelIpTextFieldStates.clear()
                        tunnelIpTextFieldStates.add(TextFieldState(""))
                        publicKeyTextFieldState.clearText()
                        allowedIpTextFieldStates.clear()
                        allowedIpTextFieldStates.add(TextFieldState(""))
                        endpointTextFieldState.clearText()
                        clearConfig()
                    },
                )
            }
        },
    ) { modifier ->
        Column(
            modifier =
                modifier
                    .verticalScroll(rememberScrollState())
                    .animateContentSize()
                    .padding(horizontal = Dimens.sideMarginNew),
            verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
        ) {
            // Scale image to fit width up to certain width
            Image(
                contentScale = ContentScale.FillWidth,
                modifier =
                    Modifier.widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                        .fillMaxWidth()
                        .align(Alignment.CenterHorizontally),
                painter = painterResource(id = R.drawable.personal_vpn_illustration),
                contentDescription = stringResource(R.string.multihop),
            )
            ScreenDescription(stringResource(R.string.personal_vpn_description))
            SwitchListItem(
                title = stringResource(R.string.enable),
                isToggled = state.value.enabled,
                onCellClicked = { onTogglePersonalVpn(!state.value.enabled) },
                position = Position.Single,
            )

            Card(
                shape = Shapes.large,
                colors =
                    CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
                border = BorderStroke(1.dp, color = MaterialTheme.colorScheme.primary),
            ) {
                Column(modifier = Modifier.padding(all = Dimens.smallPadding)) {
                    Text("Interface", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(Dimens.smallSpacer))

                    var showPassword by remember { mutableStateOf(false) }
                    SecureTextField(
                        modifier = Modifier.fillMaxWidth(),
                        state = privateKeyTextFieldState,
                        label = { Text("Private key") },
                        labelPosition = TextFieldLabelPosition.Above(),
                        isError = state.value.privateKeyDataError != null,
                        placeholder = { Text("AAAA...AAAA=") },
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
                                        if (showPassword)
                                            stringResource(id = R.string.hide_account_number)
                                        else stringResource(id = R.string.show_account_number),
                                )
                            }
                        },
                        supportingText =
                            state.value.privateKeyDataError?.let { { Text(it.toErrorMessage()) } },
                    )
                    Spacer(Modifier.height(Dimens.mediumSpacer))

                    // Tunnel addresses - dynamic list of text fields
                    Text(
                        "Addresses",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    tunnelIpTextFieldStates.forEachIndexed { index, textFieldState ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.Top,
                        ) {
                            TextField(
                                modifier = Modifier.padding(Dimens.miniPadding).weight(1f),
                                state = textFieldState,
                                placeholder = { Text("10.0.0.2") },
                                isError = state.value.tunnelIpDataErrors.containsKey(index),
                                lineLimits = TextFieldLineLimits.SingleLine,
                                colors = mullvadDarkTextFieldColors(),
                                supportingText =
                                    state.value.tunnelIpDataErrors[index]?.let {
                                        { Text(it.toErrorMessage()) }
                                    },
                                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                            )
                            if (tunnelIpTextFieldStates.size > 1) {
                                IconButton(
                                    onClick = {
                                        tunnelIpTextFieldStates.removeAt(index)
                                        onClearTunnelIpErrors()
                                    }
                                ) {
                                    Icon(
                                        imageVector = Icons.Outlined.Close,
                                        contentDescription = "Remove address",
                                    )
                                }
                            }
                        }
                    }
                    TextButton(
                        onClick = { tunnelIpTextFieldStates.add(TextFieldState("")) },
                        colors =
                            ButtonDefaults.buttonColors(
                                contentColor = MaterialTheme.colorScheme.onSurface
                            ),
                    ) {
                        Icon(imageVector = Icons.Outlined.Add, contentDescription = null)
                        Text("Add address")
                    }
                }
            }

            Card(
                shape = Shapes.large,
                colors =
                    CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
                border = BorderStroke(1.dp, color = MaterialTheme.colorScheme.primary),
            ) {
                Column(modifier = Modifier.padding(Dimens.smallPadding)) {
                    Text("Peer", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(Dimens.smallSpacer))
                    TextField(
                        modifier = Modifier.fillMaxWidth(),
                        state = publicKeyTextFieldState,
                        label = { Text("Public key") },
                        labelPosition = TextFieldLabelPosition.Above(),
                        placeholder = { Text("AAAA...AAAA=") },
                        isError = state.value.publicKeyDataError != null,
                        colors = mullvadDarkTextFieldColors(),
                        lineLimits = TextFieldLineLimits.SingleLine,
                        supportingText =
                            state.value.publicKeyDataError?.let { { Text(it.toErrorMessage()) } },
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    )
                    Spacer(Modifier.height(Dimens.mediumSpacer))

                    // Allowed IPs - dynamic list of text fields
                    Text(
                        "Allowed IPs",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    allowedIpTextFieldStates.forEachIndexed { index, textFieldState ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.Top,
                        ) {
                            TextField(
                                modifier = Modifier.padding(Dimens.miniPadding).weight(1f),
                                state = textFieldState,
                                placeholder = { Text("0.0.0.0/0") },
                                isError = state.value.allowedIpDataErrors.containsKey(index),
                                lineLimits = TextFieldLineLimits.SingleLine,
                                colors = mullvadDarkTextFieldColors(),
                                supportingText =
                                    state.value.allowedIpDataErrors[index]?.let {
                                        { Text(it.toErrorMessage()) }
                                    },
                                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                            )
                            if (allowedIpTextFieldStates.size > 1) {
                                IconButton(
                                    onClick = {
                                        allowedIpTextFieldStates.removeAt(index)
                                        onClearAllowedIpErrors()
                                    }
                                ) {
                                    Icon(
                                        imageVector = Icons.Outlined.Close,
                                        contentDescription = "Remove allowed IP",
                                    )
                                }
                            }
                        }
                    }
                    TextButton(
                        onClick = {
                            allowedIpTextFieldStates.add(TextFieldState(""))
                        },
                            colors = ButtonDefaults.buttonColors(contentColor = MaterialTheme.colorScheme.onSurface)
                    ) {
                        Icon(
                            imageVector = Icons.Outlined.Add,
                            contentDescription = null,
                        )
                        Text("Add allowed IP")
                    }

                    Spacer(Modifier.height(Dimens.mediumSpacer))
                    val keyboardController = LocalSoftwareKeyboardController.current
                    TextField(
                        modifier = Modifier.fillMaxWidth(),
                        state = endpointTextFieldState,
                        label = { Text("Endpoint") },
                        labelPosition = TextFieldLabelPosition.Above(),
                        placeholder = { Text("1.2.3.4:51820") },
                        isError = state.value.endpointDataError != null,
                        colors = mullvadDarkTextFieldColors(),
                        supportingText =
                            state.value.endpointDataError?.let { { Text(it.toErrorMessage()) } },
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                        lineLimits = TextFieldLineLimits.SingleLine,
                        onKeyboardAction = {
                            if (formHasChanges) {
                                saveConfig(
                                    PersonalVpnFormData(
                                        privateKey = privateKeyTextFieldState.text.toString(),
                                        tunnelIps =
                                            tunnelIpTextFieldStates.map { it.text.toString() },
                                        publicKey = publicKeyTextFieldState.text.toString(),
                                        allowedIPs =
                                            allowedIpTextFieldStates.map { it.text.toString() },
                                        endpoint = endpointTextFieldState.text.toString(),
                                    )
                                )
                            } else keyboardController?.hide()
                        },
                    )

                    Spacer(Modifier.height(Dimens.smallSpacer))

                    if (state.value.enabled) {
                        HorizontalDivider(color = MaterialTheme.colorScheme.surfaceContainerLow)
                        Spacer(Modifier.height(Dimens.smallSpacer))
                        val tunnelStats = state.value.tunnelStats
                        Card(
                            colors =
                                CardDefaults.cardColors(
                                    containerColor = MaterialTheme.colorScheme.surfaceContainerLow
                                )
                        ) {
                            CompositionLocalProvider(
                                LocalTextStyle provides MaterialTheme.typography.bodySmall
                            ) {
                                Column(modifier = Modifier.padding(8.dp).fillMaxWidth()) {
                                    Text(
                                        text = "Tunnel stats",
                                        style = MaterialTheme.typography.titleSmall,
                                    )
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                    ) {
                                        Text("rx:")
                                        Text("${tunnelStats.rx} bytes")
                                    }
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                    ) {
                                        Text("tx:")
                                        Text("${tunnelStats.tx} bytes")
                                    }
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                    ) {
                                        Text("last handshake:")
                                        val lastHandshakeSeconds =
                                            tunnelStats.lastHandshake?.epochSecond
                                        var secondsAgo by
                                            remember(lastHandshakeSeconds) {
                                                mutableStateOf<Long?>(null)
                                            }
                                        LaunchedEffect(tunnelStats.lastHandshake) {
                                            val lastHandshakeEpochSeconds =
                                                tunnelStats.lastHandshake?.epochSecond
                                                    ?: return@LaunchedEffect

                                            while (true) {
                                                secondsAgo =
                                                    Instant.now().epochSecond -
                                                        lastHandshakeEpochSeconds
                                                delay(1.seconds)
                                            }
                                        }
                                        val lastHandshakeString =
                                            if (lastHandshakeSeconds == null) {
                                                "Never"
                                            } else {
                                                "$secondsAgo seconds ago"
                                            }
                                        Text(lastHandshakeString)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun FormDataError.toErrorMessage(): String =
    when (this) {
        is FormDataError.AllowedIp -> "Bad allowed IP"
        is FormDataError.Endpoint -> this.toString()
        is FormDataError.PrivateKey -> keyParseError.toString()
        is FormDataError.PublicKey -> keyParseError.toString()
        is FormDataError.TunnelIp -> "Bad address IP"
    }
