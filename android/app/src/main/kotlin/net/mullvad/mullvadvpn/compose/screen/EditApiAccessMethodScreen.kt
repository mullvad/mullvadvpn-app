package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorSmall
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.destinations.SaveApiAccessMethodDestination
import net.mullvad.mullvadvpn.compose.preview.EditApiAccessMethodUiStateParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodTypes
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.state.TestMethodState
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.compose.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodInvalidDataErrors
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.menuItemColors
import net.mullvad.mullvadvpn.lib.theme.color.selected
import net.mullvad.mullvadvpn.viewmodel.EditApiAccessMethodViewModel
import net.mullvad.mullvadvpn.viewmodel.EditApiAccessSideEffect
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
fun PreviewEditApiAccessMethodScreen(
    @PreviewParameter(EditApiAccessMethodUiStateParameterProvider::class)
    state: EditApiAccessMethodUiState
) {
    AppTheme { EditApiAccessMethodScreen(state = state) }
}

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun EditApiAccessMethod(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<Boolean>,
    saveApiAccessMethodResultRecipient: ResultRecipient<SaveApiAccessMethodDestination, Boolean>,
    accessMethodId: ApiAccessMethodId?
) {
    val viewModel =
        koinViewModel<EditApiAccessMethodViewModel>(parameters = { parametersOf(accessMethodId) })

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    LaunchedEffectCollect(sideEffect = viewModel.sideEffect) {
        when (it) {
            is EditApiAccessSideEffect.OpenSaveDialog ->
                navigator.navigate(SaveApiAccessMethodDestination(it.newAccessMethod))
            EditApiAccessSideEffect.UnableToGetApiAccessMethod ->
                backNavigator.navigateBack(result = false)
        }
    }

    saveApiAccessMethodResultRecipient.OnNavResultValue { saveSuccessful ->
        if (saveSuccessful) {
            backNavigator.navigateBack(result = true)
        } else {
            // Show error snackbar
            scope.launch {
                snackbarHostState.showSnackbarImmediately(
                    message = context.getString(R.string.error_occurred)
                )
            }
        }
    }

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    EditApiAccessMethodScreen(
        state = state,
        onNameChanged = viewModel::updateName,
        onTypeSelected = viewModel::setAccessMethodType,
        onIpChanged = viewModel::updateServerIp,
        onRemotePortChanged = viewModel::updateRemotePort,
        onLocalPortChanged = viewModel::updateLocalPort,
        onPasswordChanged = viewModel::updatePassword,
        onCipherChange = viewModel::updateCipher,
        onToggleAuthenticationEnabled = viewModel::updateAuthenticationEnabled,
        onUsernameChanged = viewModel::updateUsername,
        onTransportProtocolChanged = viewModel::updateTransportProtocol,
        onTestMethod = viewModel::testMethod,
        onAddMethod = viewModel::trySave,
        onNavigateBack = { navigator.navigateUp() }
    )
}

@Composable
fun EditApiAccessMethodScreen(
    state: EditApiAccessMethodUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onNameChanged: (ApiAccessMethodName) -> Unit = {},
    onTypeSelected: (ApiAccessMethodTypes) -> Unit = {},
    onIpChanged: (String) -> Unit = {},
    onRemotePortChanged: (Port?) -> Unit = {},
    onLocalPortChanged: (Port?) -> Unit = {},
    onPasswordChanged: (String) -> Unit = {},
    onCipherChange: (Cipher) -> Unit = {},
    onToggleAuthenticationEnabled: (Boolean) -> Unit = {},
    onUsernameChanged: (String) -> Unit = {},
    onTransportProtocolChanged: (TransportProtocol) -> Unit = {},
    onTestMethod: () -> Unit = {},
    onAddMethod: () -> Unit = {},
    onNavigateBack: () -> Unit = {}
) {
    ScaffoldWithMediumTopBar(
        snackbarHostState = snackbarHostState,
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onNavigateBack) },
        appBarTitle =
            stringResource(
                if (state.editMode) {
                    R.string.edit_method
                } else {
                    R.string.add_method
                }
            ),
    ) { modifier ->
        Column(modifier = modifier.padding(horizontal = Dimens.screenVerticalMargin)) {
            when (state) {
                is EditApiAccessMethodUiState.Loading -> Loading()
                is EditApiAccessMethodUiState.Content -> {
                    NameInputField(
                        name = state.formData.name,
                        error = state.formErrors?.getErrorOrNull<InvalidDataError.NameError>(),
                        onNameChanged = onNameChanged
                    )
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    ApiAccessMethodTypeSelection(state.formData, onTypeSelected)
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    when (val formData = state.formData) {
                        is EditApiAccessFormData.Shadowsocks ->
                            ShadowsocksForm(
                                formData = formData,
                                errors = state.formErrors,
                                onIpChanged = onIpChanged,
                                onPortChanged = onRemotePortChanged,
                                onPasswordChanged = onPasswordChanged,
                                onCipherChange = onCipherChange
                            )
                        is EditApiAccessFormData.Socks5Local ->
                            Socks5LocalForm(
                                formData = formData,
                                errors = state.formErrors,
                                onLocalPortChanged = onLocalPortChanged,
                                onRemoteIpChanged = onIpChanged,
                                onRemotePortChanged = onRemotePortChanged,
                                onTransportProtocolChanged = onTransportProtocolChanged
                            )
                        is EditApiAccessFormData.Socks5Remote ->
                            Socks5RemoteForm(
                                formData = formData,
                                errors = state.formErrors,
                                onIpChanged = onIpChanged,
                                onPortChanged = onRemotePortChanged,
                                onToggleAuthenticationEnabled = onToggleAuthenticationEnabled,
                                onUsernameChanged = onUsernameChanged,
                                onPasswordChanged = onPasswordChanged
                            )
                    }
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    TestMethodButton(
                        testMethodState = state.testMethodState,
                        onTestMethod = onTestMethod
                    )
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    AddMethodButton(onAddMethod = onAddMethod)
                }
            }
        }
    }
}

@Composable private fun Loading() {}

@Composable
private fun NameInputField(
    name: ApiAccessMethodName?,
    error: InvalidDataError.NameError?,
    onNameChanged: (ApiAccessMethodName) -> Unit,
) {
    InputField(
        value = name?.value ?: "",
        keyboardType = KeyboardType.Text,
        onValueChanged = { onNameChanged(ApiAccessMethodName.fromString(it)) },
        onSubmit = {},
        placeholderText = stringResource(id = R.string.name),
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        maxCharLength = ApiAccessMethodName.MAX_LENGTH,
        supportingText =
            error?.let {
                {
                    Text(
                        text = textResource(id = R.string.this_field_is_required),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ApiAccessMethodTypeSelection(
    formData: EditApiAccessFormData,
    onTypeSelected: (ApiAccessMethodTypes) -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        TextField(
            modifier = Modifier.fillMaxWidth().menuAnchor(),
            readOnly = true,
            value = formData.text(),
            onValueChange = {},
            label = { Text(stringResource(id = R.string.type)) },
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
            colors = mullvadDarkTextFieldColors()
        )
        ExposedDropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)
        ) {
            ApiAccessMethodTypes.entries.forEach {
                DropdownMenuItem(
                    colors = menuItemColors,
                    text = { Text(text = it.text()) },
                    onClick = {
                        onTypeSelected(it)
                        expanded = false
                    }
                )
            }
        }
    }
}

@Composable
private fun ColumnScope.ShadowsocksForm(
    formData: EditApiAccessFormData.Shadowsocks,
    errors: ApiAccessMethodInvalidDataErrors?,
    onIpChanged: (String) -> Unit,
    onPortChanged: (Port?) -> Unit,
    onPasswordChanged: (String) -> Unit,
    onCipherChange: (Cipher) -> Unit
) {
    ServerIpInput(ip = formData.ip, error = errors?.getErrorOrNull(), onIpChanged = onIpChanged)
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    RemotePortInput(
        port = formData.port,
        error = errors?.getErrorOrNull(),
        onPortChanged = onPortChanged
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    PasswordInput(
        password = formData.password,
        optional = true,
        onPasswordChanged = onPasswordChanged
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    CipherSelection(cipher = formData.cipher, onCipherChange = onCipherChange)
}

@Composable
private fun ColumnScope.Socks5LocalForm(
    formData: EditApiAccessFormData.Socks5Local,
    errors: ApiAccessMethodInvalidDataErrors?,
    onLocalPortChanged: (Port?) -> Unit,
    onRemoteIpChanged: (String) -> Unit,
    onRemotePortChanged: (Port?) -> Unit,
    onTransportProtocolChanged: (TransportProtocol) -> Unit
) {
    LocalPortInput(
        port = formData.localPort,
        error = errors?.getErrorOrNull(),
        onPortChanged = onLocalPortChanged
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
    ServerIpInput(
        ip = formData.remoteIp,
        error = errors?.getErrorOrNull(),
        onIpChanged = onRemoteIpChanged
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    RemotePortInput(
        port = formData.remotePort,
        error = errors?.getErrorOrNull(),
        onPortChanged = onRemotePortChanged
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    TransportProtocolSelect(
        transportProtocol = formData.remoteTransportProtocol,
        onTransportProtocolChanged = onTransportProtocolChanged
    )
}

@Composable
private fun ColumnScope.Socks5RemoteForm(
    formData: EditApiAccessFormData.Socks5Remote,
    errors: ApiAccessMethodInvalidDataErrors?,
    onIpChanged: (String) -> Unit,
    onPortChanged: (Port?) -> Unit,
    onToggleAuthenticationEnabled: (Boolean) -> Unit,
    onUsernameChanged: (String) -> Unit,
    onPasswordChanged: (String) -> Unit
) {
    ServerIpInput(ip = formData.ip, error = errors?.getErrorOrNull(), onIpChanged = onIpChanged)
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    RemotePortInput(
        port = formData.port,
        error = errors?.getErrorOrNull(),
        onPortChanged = onPortChanged
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    EnableAuthentication(formData.enableAuthentication, onToggleAuthenticationEnabled)
    if (formData.enableAuthentication) {
        Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
        UsernameInput(
            username = formData.username,
            error = errors?.getErrorOrNull(),
            onUsernameChanged = onUsernameChanged
        )
        Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
        PasswordInput(
            password = formData.password,
            error = errors?.getErrorOrNull(),
            optional = false,
            onPasswordChanged = onPasswordChanged
        )
    }
}

@Composable
private fun ServerIpInput(
    ip: String?,
    error: InvalidDataError.ServerIpError?,
    onIpChanged: (String) -> Unit
) {
    InputField(
        value = ip ?: "",
        keyboardType = KeyboardType.Text,
        onValueChanged = onIpChanged,
        onSubmit = {},
        placeholderText = stringResource(id = R.string.server),
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        supportingText =
            error?.let {
                {
                    Text(
                        text =
                            textResource(
                                id =
                                    when (it) {
                                        InvalidDataError.ServerIpError.Invalid ->
                                            R.string.please_enter_a_valid_ip_address
                                        InvalidDataError.ServerIpError.Required ->
                                            R.string.this_field_is_required
                                    }
                            ),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            },
    )
}

@Composable
private fun RemotePortInput(
    port: Port?,
    error: InvalidDataError.RemotePortError?,
    onPortChanged: (Port?) -> Unit
) {
    InputField(
        value = port?.value?.toString() ?: "",
        keyboardType = KeyboardType.Number,
        onValueChanged = { onPortChanged(it.toPortOrNull()) },
        onSubmit = {},
        placeholderText = stringResource(id = R.string.port),
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        supportingText =
            error?.let {
                {
                    Text(
                        text =
                            textResource(
                                id =
                                    when (it) {
                                        InvalidDataError.RemotePortError.Invalid ->
                                            R.string.please_enter_a_valid_remote_server_port
                                        InvalidDataError.RemotePortError.Required ->
                                            R.string.this_field_is_required
                                    }
                            ),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            },
    )
}

@Composable
private fun LocalPortInput(
    port: Port?,
    error: InvalidDataError.RemotePortError?,
    onPortChanged: (Port?) -> Unit
) {
    InputField(
        value = port?.value?.toString() ?: "",
        keyboardType = KeyboardType.Number,
        onValueChanged = { onPortChanged(it.toPortOrNull()) },
        onSubmit = {},
        placeholderText = stringResource(id = R.string.port),
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        supportingText =
            error?.let {
                {
                    Text(
                        text =
                            textResource(
                                id =
                                    when (it) {
                                        InvalidDataError.RemotePortError.Invalid ->
                                            R.string.please_enter_a_valid_localhost_port
                                        InvalidDataError.RemotePortError.Required ->
                                            R.string.this_field_is_required
                                    }
                            ),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            },
    )
}

@Composable
private fun PasswordInput(
    password: String?,
    optional: Boolean,
    error: InvalidDataError.PasswordError? = null,
    onPasswordChanged: (String) -> Unit
) {
    InputField(
        value = password ?: "",
        keyboardType = KeyboardType.Password,
        onValueChanged = onPasswordChanged,
        onSubmit = {},
        placeholderText =
            stringResource(
                id =
                    if (optional) {
                        R.string.password_optional
                    } else {
                        R.string.password
                    }
            ),
        isValidValue = true,
        isDigitsOnlyAllowed = false,
        supportingText =
            error?.let {
                {
                    Text(
                        text = textResource(id = R.string.this_field_is_required),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CipherSelection(cipher: Cipher, onCipherChange: (Cipher) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        TextField(
            modifier = Modifier.fillMaxWidth().menuAnchor(),
            readOnly = true,
            value = cipher.value,
            onValueChange = {},
            label = { Text(stringResource(id = R.string.cipher)) },
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
            colors = mullvadDarkTextFieldColors(),
        )
        ExposedDropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)
        ) {
            Cipher.listAll().forEach {
                DropdownMenuItem(
                    colors = menuItemColors,
                    text = { Text(text = it.value) },
                    onClick = {
                        onCipherChange(it)
                        expanded = false
                    },
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EnableAuthentication(
    authenticationEnabled: Boolean,
    onToggleAuthenticationEnabled: (Boolean) -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        TextField(
            modifier = Modifier.fillMaxWidth().menuAnchor(),
            readOnly = true,
            value =
                stringResource(
                    id =
                        if (authenticationEnabled) {
                            R.string.on
                        } else {
                            R.string.off
                        }
                ),
            onValueChange = {},
            label = { Text(stringResource(id = R.string.authentication)) },
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
            colors = mullvadDarkTextFieldColors(),
        )
        ExposedDropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)
        ) {
            DropdownMenuItem(
                colors = menuItemColors,
                text = { Text(text = stringResource(id = R.string.on)) },
                onClick = {
                    onToggleAuthenticationEnabled(true)
                    expanded = false
                },
            )
            DropdownMenuItem(
                colors = menuItemColors,
                text = { Text(text = stringResource(id = R.string.off)) },
                onClick = {
                    onToggleAuthenticationEnabled(false)
                    expanded = false
                },
            )
        }
    }
}

@Composable
private fun UsernameInput(
    username: String?,
    error: InvalidDataError.UserNameError?,
    onUsernameChanged: (String) -> Unit,
) {
    InputField(
        value = username ?: "",
        keyboardType = KeyboardType.Text,
        onValueChanged = onUsernameChanged,
        onSubmit = {},
        placeholderText = stringResource(id = R.string.username),
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        supportingText =
            error?.let {
                {
                    Text(
                        text = textResource(id = R.string.this_field_is_required),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TransportProtocolSelect(
    transportProtocol: TransportProtocol,
    onTransportProtocolChanged: (TransportProtocol) -> Unit
) {
    var expanded by remember { mutableStateOf(false) }
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        TextField(
            modifier = Modifier.fillMaxWidth().menuAnchor(),
            readOnly = true,
            value = transportProtocol.text(),
            onValueChange = {},
            label = { Text(stringResource(id = R.string.transport_protocol)) },
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
            colors = mullvadDarkTextFieldColors()
        )
        ExposedDropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)
        ) {
            TransportProtocol.entries.forEach {
                DropdownMenuItem(
                    colors = menuItemColors,
                    text = { Text(text = it.text()) },
                    onClick = {
                        onTransportProtocolChanged(it)
                        expanded = false
                    },
                )
            }
        }
    }
}

@Composable
private fun TestMethodButton(testMethodState: TestMethodState?, onTestMethod: () -> Unit) {
    PrimaryButton(
        leadingIcon =
            testMethodState?.let {
                {
                    when (testMethodState) {
                        TestMethodState.Failed ->
                            Box(
                                modifier =
                                    Modifier.size(Dimens.relayCircleSize)
                                        .background(
                                            color = MaterialTheme.colorScheme.error,
                                            shape = CircleShape
                                        )
                            )
                        TestMethodState.Successful -> {
                            Box(
                                modifier =
                                    Modifier.size(Dimens.relayCircleSize)
                                        .background(
                                            color = MaterialTheme.colorScheme.selected,
                                            shape = CircleShape
                                        )
                            )
                        }
                        TestMethodState.Testing -> {
                            MullvadCircularProgressIndicatorSmall()
                        }
                    }
                }
            },
        onClick = onTestMethod,
        text =
            stringResource(
                id =
                    when (testMethodState) {
                        TestMethodState.Successful -> R.string.api_reachable
                        TestMethodState.Failed -> R.string.api_unreachable
                        TestMethodState.Testing -> R.string.testing
                        null -> R.string.test_method
                    }
            )
    )
}

@Composable
private fun AddMethodButton(onAddMethod: () -> Unit) {
    PrimaryButton(onClick = onAddMethod, text = stringResource(id = R.string.add))
}

@Composable
private fun InputField(
    value: String,
    keyboardType: KeyboardType,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    onSubmit: (String) -> Unit,
    placeholderText: String?,
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: Boolean,
    isDigitsOnlyAllowed: Boolean,
    supportingText: @Composable (() -> Unit)? = null,
) {
    CustomTextField(
        value = value,
        keyboardType = keyboardType,
        onValueChanged = onValueChanged,
        onSubmit = onSubmit,
        placeholderText = placeholderText,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = isDigitsOnlyAllowed,
        maxCharLength = maxCharLength,
        supportingText = supportingText
    )
}

@Composable
private fun EditApiAccessFormData.text(): String =
    stringResource(
        id =
            when (this) {
                is EditApiAccessFormData.Socks5Local -> R.string.socks5_local
                is EditApiAccessFormData.Socks5Remote -> R.string.socks5_remote
                is EditApiAccessFormData.Shadowsocks -> R.string.shadowsocks
            }
    )

@Composable
private fun ApiAccessMethodTypes.text(): String =
    stringResource(
        id =
            when (this) {
                ApiAccessMethodTypes.SHADOWSOCKS -> R.string.shadowsocks
                ApiAccessMethodTypes.SOCKS5_LOCAL -> R.string.socks5_local
                ApiAccessMethodTypes.SOCKS5_REMOTE -> R.string.socks5_remote
            },
    )

@Composable
private fun TransportProtocol.text(): String =
    when (this) {
        TransportProtocol.Tcp -> stringResource(id = R.string.tcp)
        TransportProtocol.Udp -> stringResource(id = R.string.udp)
    }

fun String.toPortOrNull() = toIntOrNull()?.let { Port(it) }
