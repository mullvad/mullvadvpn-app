package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import net.mullvad.mullvadvpn.compose.button.TestMethodButton
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.destinations.DiscardChangesDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.SaveApiAccessMethodDestination
import net.mullvad.mullvadvpn.compose.preview.EditApiAccessMethodUiStateParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodTypes
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.state.FormInputField
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.compose.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.InvalidDataError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.menuItemColors
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
    discardCangesResultRecipient: ResultRecipient<DiscardChangesDialogDestination, Boolean>,
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
                navigator.navigate(SaveApiAccessMethodDestination(it.newAccessMethod)) {
                    launchSingleTop = true
                }
            is EditApiAccessSideEffect.UnableToGetApiAccessMethod ->
                backNavigator.navigateBack(result = false)
            EditApiAccessSideEffect.CLoseScreen -> navigator.navigateUp()
            EditApiAccessSideEffect.ShowDiscardChangesDialog ->
                navigator.navigate(DiscardChangesDialogDestination) { launchSingleTop = true }
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

    discardCangesResultRecipient.OnNavResultValue { discardChanges ->
        if (discardChanges) {
            navigator.navigateUp()
        }
    }

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    EditApiAccessMethodScreen(
        state = state,
        onNameChanged = viewModel::updateName,
        onTypeSelected = viewModel::setAccessMethodType,
        onIpChanged = viewModel::updateServerIp,
        onRemotePortChanged = viewModel::updateRemotePort,
        onPasswordChanged = viewModel::updatePassword,
        onCipherChange = viewModel::updateCipher,
        onToggleAuthenticationEnabled = viewModel::updateAuthenticationEnabled,
        onUsernameChanged = viewModel::updateUsername,
        onTestMethod = viewModel::testMethod,
        onAddMethod = viewModel::trySave,
        onNavigateBack = viewModel::onNavigateBack
    )
}

@Composable
fun EditApiAccessMethodScreen(
    state: EditApiAccessMethodUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onNameChanged: (String) -> Unit = {},
    onTypeSelected: (ApiAccessMethodTypes) -> Unit = {},
    onIpChanged: (String) -> Unit = {},
    onRemotePortChanged: (String) -> Unit = {},
    onPasswordChanged: (String) -> Unit = {},
    onCipherChange: (Cipher) -> Unit = {},
    onToggleAuthenticationEnabled: (Boolean) -> Unit = {},
    onUsernameChanged: (String) -> Unit = {},
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
                        nameFormData = state.formData.name,
                        onNameChanged = onNameChanged
                    )
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    ApiAccessMethodTypeSelection(state.formData, onTypeSelected)
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    when (state.formData.apiAccessMethodTypes) {
                        ApiAccessMethodTypes.SHADOWSOCKS ->
                            ShadowsocksForm(
                                formData = state.formData,
                                onIpChanged = onIpChanged,
                                onPortChanged = onRemotePortChanged,
                                onPasswordChanged = onPasswordChanged,
                                onCipherChange = onCipherChange
                            )
                        ApiAccessMethodTypes.SOCKS5_REMOTE ->
                            Socks5RemoteForm(
                                formData = state.formData,
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
                    AddMethodButton(isNew = !state.editMode, onAddMethod = onAddMethod)
                }
            }
        }
    }
}

@Composable private fun Loading() {}

@Composable
private fun NameInputField(
    nameFormData: FormInputField<InvalidDataError.NameError>,
    onNameChanged: (String) -> Unit,
) {
    InputField(
        value = nameFormData.input,
        keyboardType = KeyboardType.Text,
        onValueChanged = onNameChanged,
        onSubmit = {},
        placeholderText = stringResource(id = R.string.name),
        isValidValue = nameFormData.error == null,
        isDigitsOnlyAllowed = false,
        maxCharLength = ApiAccessMethodName.MAX_LENGTH,
        supportingText =
            nameFormData.error?.let {
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
            value = formData.apiAccessMethodTypes.text(),
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
private fun ShadowsocksForm(
    formData: EditApiAccessFormData,
    onIpChanged: (String) -> Unit,
    onPortChanged: (String) -> Unit,
    onPasswordChanged: (String) -> Unit,
    onCipherChange: (Cipher) -> Unit
) {
    ServerIpInput(ipFormData = formData.ip, onIpChanged = onIpChanged)
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    RemotePortInput(portFormData = formData.remotePort, onPortChanged = onPortChanged)
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    PasswordInput(
        passwordFormData = formData.password,
        optional = true,
        onPasswordChanged = onPasswordChanged
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    CipherSelection(cipher = formData.cipher, onCipherChange = onCipherChange)
}

@Composable
private fun Socks5RemoteForm(
    formData: EditApiAccessFormData,
    onIpChanged: (String) -> Unit,
    onPortChanged: (String) -> Unit,
    onToggleAuthenticationEnabled: (Boolean) -> Unit,
    onUsernameChanged: (String) -> Unit,
    onPasswordChanged: (String) -> Unit
) {
    ServerIpInput(ipFormData = formData.ip, onIpChanged = onIpChanged)
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    RemotePortInput(portFormData = formData.remotePort, onPortChanged = onPortChanged)
    Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
    EnableAuthentication(formData.enableAuthentication, onToggleAuthenticationEnabled)
    if (formData.enableAuthentication) {
        Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
        UsernameInput(usernameFormData = formData.username, onUsernameChanged = onUsernameChanged)
        Spacer(modifier = Modifier.height(Dimens.verticalSpacer))
        PasswordInput(
            passwordFormData = formData.password,
            optional = false,
            onPasswordChanged = onPasswordChanged
        )
    }
}

@Composable
private fun ServerIpInput(
    ipFormData: FormInputField<InvalidDataError.ServerIpError>,
    onIpChanged: (String) -> Unit
) {
    InputField(
        value = ipFormData.input,
        keyboardType = KeyboardType.Text,
        onValueChanged = onIpChanged,
        onSubmit = {},
        placeholderText = stringResource(id = R.string.server),
        isValidValue = ipFormData.error == null,
        isDigitsOnlyAllowed = false,
        supportingText =
            ipFormData.error?.let {
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
    portFormData: FormInputField<InvalidDataError.RemotePortError>,
    onPortChanged: (String) -> Unit
) {
    InputField(
        value = portFormData.input,
        keyboardType = KeyboardType.Number,
        onValueChanged = onPortChanged,
        onSubmit = {},
        placeholderText = stringResource(id = R.string.port),
        isValidValue = portFormData.error == null,
        isDigitsOnlyAllowed = false,
        supportingText =
            portFormData.error?.let {
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
private fun PasswordInput(
    passwordFormData: FormInputField<InvalidDataError.PasswordError>,
    optional: Boolean,
    onPasswordChanged: (String) -> Unit
) {
    InputField(
        value = passwordFormData.input,
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
            passwordFormData.error?.let {
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
            value = cipher.label,
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
                    text = { Text(text = it.label) },
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
    usernameFormData: FormInputField<InvalidDataError.UserNameError>,
    onUsernameChanged: (String) -> Unit,
) {
    InputField(
        value = usernameFormData.input,
        keyboardType = KeyboardType.Text,
        onValueChanged = onUsernameChanged,
        onSubmit = {},
        placeholderText = stringResource(id = R.string.username),
        isValidValue = usernameFormData.error == null,
        isDigitsOnlyAllowed = false,
        supportingText =
            usernameFormData.error?.let {
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

@Composable
private fun AddMethodButton(isNew: Boolean, onAddMethod: () -> Unit) {
    PrimaryButton(
        onClick = onAddMethod,
        text =
            stringResource(
                id =
                    if (isNew) {
                        R.string.add
                    } else {
                        R.string.save
                    }
            )
    )
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
private fun ApiAccessMethodTypes.text(): String =
    stringResource(
        id =
            when (this) {
                ApiAccessMethodTypes.SHADOWSOCKS -> R.string.shadowsocks
                ApiAccessMethodTypes.SOCKS5_REMOTE -> R.string.socks5_remote
            },
    )
