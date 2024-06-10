package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
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
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.MullvadDropdownMenuItem
import net.mullvad.mullvadvpn.compose.component.MullvadExposedDropdownMenuBox
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.destinations.DiscardChangesDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.SaveApiAccessMethodDestination
import net.mullvad.mullvadvpn.compose.preview.EditApiAccessMethodUiStateParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodTypes
import net.mullvad.mullvadvpn.compose.state.EditApiAccessFormData
import net.mullvad.mullvadvpn.compose.state.EditApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.textfield.ApiAccessMethodTextField
import net.mullvad.mullvadvpn.compose.textfield.apiAccessTextFieldColors
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
                navigator.navigate(
                    SaveApiAccessMethodDestination(
                        id = it.id,
                        name = it.name,
                        enabled = it.enabled,
                        customProxy = it.customProxy
                    )
                ) {
                    launchSingleTop = true
                }
            is EditApiAccessSideEffect.UnableToGetApiAccessMethod ->
                backNavigator.navigateBack(result = false)
            EditApiAccessSideEffect.CloseScreen -> navigator.navigateUp()
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
        onPortChanged = viewModel::updatePort,
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
    onPortChanged: (String) -> Unit = {},
    onPasswordChanged: (String) -> Unit = {},
    onCipherChange: (Cipher) -> Unit = {},
    onToggleAuthenticationEnabled: (Boolean) -> Unit = {},
    onUsernameChanged: (String) -> Unit = {},
    onTestMethod: () -> Unit = {},
    onAddMethod: () -> Unit = {},
    onNavigateBack: () -> Unit = {}
) {
    ScaffoldWithSmallTopBar(
        snackbarHostState = snackbarHostState,
        navigationIcon = { NavigateCloseIconButton(onNavigateClose = onNavigateBack) },
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
                        nameError = state.formData.nameError,
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
                                onPortChanged = onPortChanged,
                                onPasswordChanged = onPasswordChanged,
                                onCipherChange = onCipherChange
                            )
                        ApiAccessMethodTypes.SOCKS5_REMOTE ->
                            Socks5RemoteForm(
                                formData = state.formData,
                                onIpChanged = onIpChanged,
                                onPortChanged = onPortChanged,
                                onToggleAuthenticationEnabled = onToggleAuthenticationEnabled,
                                onUsernameChanged = onUsernameChanged,
                                onPasswordChanged = onPasswordChanged
                            )
                    }
                    Spacer(modifier = Modifier.height(Dimens.largePadding))
                    TestMethodButton(
                        modifier = Modifier.padding(bottom = Dimens.verticalSpace),
                        testMethodState = state.testMethodState,
                        onTestMethod = onTestMethod
                    )
                    AddMethodButton(isNew = !state.editMode, onAddMethod = onAddMethod)
                }
            }
        }
    }
}

@Composable
private fun ColumnScope.Loading() {
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
}

@Composable
private fun NameInputField(
    name: String,
    nameError: InvalidDataError.NameError?,
    onNameChanged: (String) -> Unit
) {
    val focusManager = LocalFocusManager.current
    ApiAccessMethodTextField(
        value = name,
        keyboardType = KeyboardType.Text,
        onValueChanged = onNameChanged,
        labelText = stringResource(id = R.string.name),
        isValidValue = nameError == null,
        isDigitsOnlyAllowed = false,
        maxCharLength = ApiAccessMethodName.MAX_LENGTH,
        errorText = nameError?.let { textResource(id = R.string.this_field_is_required) },
        onSubmit = { focusManager.moveFocus(FocusDirection.Down) }
    )
}

@Composable
private fun ApiAccessMethodTypeSelection(
    formData: EditApiAccessFormData,
    onTypeSelected: (ApiAccessMethodTypes) -> Unit
) {
    MullvadExposedDropdownMenuBox(
        modifier = Modifier.padding(vertical = Dimens.miniPadding),
        label = stringResource(id = R.string.type),
        title = formData.apiAccessMethodTypes.text(),
        colors = apiAccessTextFieldColors()
    ) { close ->
        ApiAccessMethodTypes.entries.forEach {
            MullvadDropdownMenuItem(
                text = it.text(),
                onClick = {
                    close()
                    onTypeSelected(it)
                }
            )
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
    val focusManager = LocalFocusManager.current
    ServerIpInput(
        serverIp = formData.serverIp,
        serverIpError = formData.serverIpError,
        onIpChanged = onIpChanged,
        onSubmit = { focusManager.moveFocus(FocusDirection.Down) }
    )
    PortInput(
        port = formData.port,
        formData.portError,
        onPortChanged = onPortChanged,
        onSubmit = { focusManager.moveFocus(FocusDirection.Down) }
    )
    PasswordInput(
        password = formData.password,
        passwordError = formData.passwordError,
        optional = true,
        onPasswordChanged = onPasswordChanged,
        onSubmit = { focusManager.clearFocus() }
    )
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
    val focusManager = LocalFocusManager.current
    ServerIpInput(
        serverIp = formData.serverIp,
        serverIpError = formData.serverIpError,
        onIpChanged = onIpChanged,
        onSubmit = { focusManager.moveFocus(FocusDirection.Down) }
    )
    PortInput(
        port = formData.port,
        portError = formData.portError,
        onPortChanged = onPortChanged,
        onSubmit = {
            if (formData.enableAuthentication) {
                focusManager.moveFocus(FocusDirection.Down)
            } else {
                focusManager.clearFocus()
            }
        }
    )
    EnableAuthentication(formData.enableAuthentication, onToggleAuthenticationEnabled)
    if (formData.enableAuthentication) {
        UsernameInput(
            username = formData.username,
            usernameError = formData.usernameError,
            onUsernameChanged = onUsernameChanged,
            onSubmit = { focusManager.moveFocus(FocusDirection.Down) }
        )
        PasswordInput(
            password = formData.password,
            passwordError = formData.passwordError,
            optional = false,
            onPasswordChanged = onPasswordChanged,
            onSubmit = { focusManager.moveFocus(FocusDirection.Down) }
        )
    }
}

@Composable
private fun ServerIpInput(
    serverIp: String,
    serverIpError: InvalidDataError.ServerIpError?,
    onIpChanged: (String) -> Unit,
    onSubmit: (String) -> Unit
) {
    ApiAccessMethodTextField(
        value = serverIp,
        keyboardType = KeyboardType.Text,
        onValueChanged = onIpChanged,
        labelText = stringResource(id = R.string.server),
        isValidValue = serverIpError == null,
        isDigitsOnlyAllowed = false,
        onSubmit = onSubmit,
        errorText =
            serverIpError?.let {
                textResource(
                    id =
                        when (it) {
                            InvalidDataError.ServerIpError.Invalid ->
                                R.string.please_enter_a_valid_ip_address
                            InvalidDataError.ServerIpError.Required ->
                                R.string.this_field_is_required
                        }
                )
            }
    )
}

@Composable
private fun PortInput(
    port: String,
    portError: InvalidDataError.PortError?,
    onPortChanged: (String) -> Unit,
    onSubmit: (String) -> Unit
) {
    ApiAccessMethodTextField(
        value = port,
        keyboardType = KeyboardType.Number,
        onValueChanged = onPortChanged,
        labelText = stringResource(id = R.string.port),
        isValidValue = portError == null,
        isDigitsOnlyAllowed = false,
        onSubmit = onSubmit,
        errorText =
            portError?.let {
                textResource(
                    id =
                        when (it) {
                            is InvalidDataError.PortError.Invalid ->
                                R.string.please_enter_a_valid_remote_server_port
                            InvalidDataError.PortError.Required -> R.string.this_field_is_required
                        }
                )
            },
    )
}

@Composable
private fun PasswordInput(
    password: String,
    passwordError: InvalidDataError.PasswordError?,
    optional: Boolean,
    onPasswordChanged: (String) -> Unit,
    onSubmit: (String) -> Unit
) {
    ApiAccessMethodTextField(
        value = password,
        keyboardType = KeyboardType.Password,
        onValueChanged = onPasswordChanged,
        labelText =
            stringResource(
                id =
                    if (optional) {
                        R.string.password_optional
                    } else {
                        R.string.password
                    }
            ),
        isValidValue = passwordError == null,
        isDigitsOnlyAllowed = false,
        onSubmit = onSubmit,
        errorText = passwordError?.let { textResource(id = R.string.this_field_is_required) },
    )
}

@Composable
private fun CipherSelection(cipher: Cipher, onCipherChange: (Cipher) -> Unit) {
    MullvadExposedDropdownMenuBox(
        modifier = Modifier.padding(vertical = Dimens.miniPadding),
        label = stringResource(id = R.string.cipher),
        title = cipher.label,
        colors = apiAccessTextFieldColors()
    ) { close ->
        Cipher.listAll().forEach {
            MullvadDropdownMenuItem(
                text = it.label,
                onClick = {
                    close()
                    onCipherChange(it)
                }
            )
        }
    }
}

@Composable
private fun EnableAuthentication(
    authenticationEnabled: Boolean,
    onToggleAuthenticationEnabled: (Boolean) -> Unit
) {
    MullvadExposedDropdownMenuBox(
        modifier = Modifier.padding(vertical = Dimens.miniPadding),
        label = stringResource(id = R.string.authentication),
        title =
            stringResource(
                id =
                    if (authenticationEnabled) {
                        R.string.on
                    } else {
                        R.string.off
                    }
            ),
        colors = apiAccessTextFieldColors()
    ) { close ->
        MullvadDropdownMenuItem(
            text = stringResource(id = R.string.on),
            onClick = {
                close()
                onToggleAuthenticationEnabled(true)
            },
        )
        MullvadDropdownMenuItem(
            text = stringResource(id = R.string.off),
            onClick = {
                close()
                onToggleAuthenticationEnabled(false)
            },
        )
    }
}

@Composable
private fun UsernameInput(
    username: String,
    usernameError: InvalidDataError.UserNameError?,
    onUsernameChanged: (String) -> Unit,
    onSubmit: (String) -> Unit
) {
    ApiAccessMethodTextField(
        value = username,
        keyboardType = KeyboardType.Text,
        onValueChanged = onUsernameChanged,
        labelText = stringResource(id = R.string.username),
        isValidValue = usernameError == null,
        isDigitsOnlyAllowed = false,
        onSubmit = onSubmit,
        errorText = usernameError?.let { textResource(id = R.string.this_field_is_required) },
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
private fun ApiAccessMethodTypes.text(): String =
    stringResource(
        id =
            when (this) {
                ApiAccessMethodTypes.SHADOWSOCKS -> R.string.shadowsocks
                ApiAccessMethodTypes.SOCKS5_REMOTE -> R.string.socks5_remote
            },
    )
