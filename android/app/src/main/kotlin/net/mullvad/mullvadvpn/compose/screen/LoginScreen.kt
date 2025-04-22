package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.destinations.CreateAccountConfirmationDestination
import com.ramcosta.composedestinations.generated.destinations.DeviceListDestination
import com.ramcosta.composedestinations.generated.destinations.OutOfTimeDestination
import com.ramcosta.composedestinations.generated.destinations.SettingsDestination
import com.ramcosta.composedestinations.generated.destinations.WelcomeDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.dialog.info.Confirmed
import net.mullvad.mullvadvpn.compose.preview.LoginUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.LoginError
import net.mullvad.mullvadvpn.compose.state.LoginState
import net.mullvad.mullvadvpn.compose.state.LoginState.Idle
import net.mullvad.mullvadvpn.compose.state.LoginState.Loading
import net.mullvad.mullvadvpn.compose.state.LoginState.Success
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.compose.test.LOGIN_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LOGIN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.compose.transitions.LoginTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.accountNumberKeyboardType
import net.mullvad.mullvadvpn.compose.util.accountNumberVisualTransformation
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.LoginUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.LoginViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Default|Loading.LoggingIn|Loading.CreatingAccount|LoginError|Success")
@Composable
private fun PreviewLoginScreen(
    @PreviewParameter(LoginUiStatePreviewParameterProvider::class) state: LoginUiState
) {
    AppTheme { LoginScreen(state = state, SnackbarHostState(), {}, {}, {}, {}, {}) }
}

private const val TOP_SPACER_WEIGHT = 1f
private const val BOTTOM_SPACER_WEIGHT = 3f

@Destination<RootGraph>(style = LoginTransition::class)
@Composable
fun Login(
    navigator: DestinationsNavigator,
    accountNumber: String? = null,
    vm: LoginViewModel = koinViewModel(),
    createAccountConfirmationDialogResult:
        ResultRecipient<CreateAccountConfirmationDestination, Confirmed>,
) {
    val state by vm.uiState.collectAsStateWithLifecycle()

    // Login with argument, e.g when user comes from Too Many Devices screen
    LaunchedEffect(accountNumber) {
        if (accountNumber != null) {
            vm.onAccountNumberChange(accountNumber)
            vm.login(accountNumber)
        }
    }

    createAccountConfirmationDialogResult.OnNavResultValue { vm.onCreateAccountConfirmed() }

    val context = LocalContext.current
    val snackbarHostState = remember { SnackbarHostState() }
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            LoginUiSideEffect.NavigateToWelcome ->
                navigator.navigate(WelcomeDestination) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
            is LoginUiSideEffect.NavigateToConnect ->
                navigator.navigate(ConnectDestination) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
            is LoginUiSideEffect.TooManyDevices ->
                navigator.navigate(DeviceListDestination(it.accountNumber)) {
                    launchSingleTop = true
                }
            LoginUiSideEffect.NavigateToOutOfTime ->
                navigator.navigate(OutOfTimeDestination) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
            LoginUiSideEffect.NavigateToCreateAccountConfirmation ->
                navigator.navigate(CreateAccountConfirmationDestination)
            LoginUiSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(
                    message = context.getString(R.string.error_occurred)
                )
        }
    }
    LoginScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onLoginClick = vm::login,
        onCreateAccountClick = vm::onCreateAccountClick,
        onDeleteHistoryClick = vm::clearAccountHistory,
        onAccountNumberChange = vm::onAccountNumberChange,
        onSettingsClick = dropUnlessResumed { navigator.navigate(SettingsDestination) },
    )
}

@Composable
private fun LoginScreen(
    state: LoginUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onLoginClick: (String) -> Unit,
    onCreateAccountClick: () -> Unit,
    onDeleteHistoryClick: () -> Unit,
    onAccountNumberChange: (String) -> Unit,
    onSettingsClick: () -> Unit,
) {
    ScaffoldWithTopBar(
        snackbarHostState = snackbarHostState,
        topBarColor = MaterialTheme.colorScheme.primary,
        iconTintColor = MaterialTheme.colorScheme.onPrimary,
        onSettingsClicked = onSettingsClick,
        enabled = state.loginState is Idle,
        onAccountClicked = null,
    ) {
        val scrollState = rememberScrollState()
        Column(
            modifier =
                Modifier.padding(it)
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.primary)
                    .verticalScroll(scrollState)
        ) {
            Spacer(modifier = Modifier.weight(TOP_SPACER_WEIGHT))
            LoginIcon(
                state.loginState,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(bottom = Dimens.largePadding),
            )
            LoginContent(state, onAccountNumberChange, onLoginClick, onDeleteHistoryClick)
            Spacer(modifier = Modifier.weight(BOTTOM_SPACER_WEIGHT))
            CreateAccountPanel(onCreateAccountClick, isEnabled = state.loginState is Idle)
        }
    }
}

@Composable
private fun LoginContent(
    state: LoginUiState,
    onAccountNumberChange: (String) -> Unit,
    onLoginClick: (String) -> Unit,
    onDeleteHistoryClick: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.sideMargin)) {
        Text(
            text = state.loginState.title(),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier =
                Modifier.testTag(LOGIN_TITLE_TEST_TAG)
                    .fillMaxWidth()
                    .padding(bottom = Dimens.smallPadding),
        )

        LoginInput(state, onLoginClick, onAccountNumberChange, onDeleteHistoryClick)

        Spacer(modifier = Modifier.size(Dimens.largePadding))
        VariantButton(
            isEnabled = state.loginButtonEnabled,
            onClick = { onLoginClick(state.accountNumberInput) },
            text = stringResource(id = R.string.login_title),
            modifier = Modifier.padding(bottom = Dimens.mediumPadding),
        )
    }
}

@Composable
@OptIn(ExperimentalComposeUiApi::class)
private fun ColumnScope.LoginInput(
    state: LoginUiState,
    onLoginClick: (String) -> Unit,
    onAccountNumberChange: (String) -> Unit,
    onDeleteHistoryClick: () -> Unit,
) {
    Text(
        modifier = Modifier.padding(bottom = Dimens.smallPadding),
        text = state.loginState.supportingText() ?: "",
        style = MaterialTheme.typography.labelMedium,
        color =
            if (state.loginState.isError()) {
                MaterialTheme.colorScheme.error
            } else {
                MaterialTheme.colorScheme.onPrimary
            },
    )

    TextField(
        modifier =
            // Fix for DPad navigation
            Modifier.focusProperties {
                    left = FocusRequester.Cancel
                    right = FocusRequester.Cancel
                }
                .fillMaxWidth()
                .testTag(LOGIN_INPUT_TEST_TAG)
                .let {
                    if (state.lastUsedAccount == null) {
                        it.clip(MaterialTheme.shapes.small)
                    } else {
                        it
                    }
                },
        value = state.accountNumberInput,
        label = {
            Text(
                text = stringResource(id = R.string.login_description),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        keyboardActions = KeyboardActions(onDone = { onLoginClick(state.accountNumberInput) }),
        keyboardOptions =
            KeyboardOptions(
                imeAction = if (state.loginButtonEnabled) ImeAction.Done else ImeAction.None,
                keyboardType = KeyboardType.accountNumberKeyboardType(LocalContext.current),
            ),
        onValueChange = onAccountNumberChange,
        singleLine = true,
        maxLines = 1,
        visualTransformation = accountNumberVisualTransformation(),
        enabled = state.loginState is Idle,
        colors = mullvadWhiteTextFieldColors(),
        isError = state.loginState.isError(),
    )

    AnimatedVisibility(visible = state.lastUsedAccount != null && state.loginState is Idle) {
        val token = state.lastUsedAccount?.value.orEmpty()
        val accountTransformation = remember { accountNumberVisualTransformation() }
        val transformedText =
            remember(token) { accountTransformation.filter(AnnotatedString(token)).text }

        AccountDropDownItem(
            accountNumber = transformedText.toString(),
            onClick = {
                state.lastUsedAccount?.let {
                    onAccountNumberChange(it.value)
                    onLoginClick(it.value)
                }
            },
            enabled = state.loginState is Idle,
            onDeleteClick = onDeleteHistoryClick,
        )
    }
}

@Composable
private fun LoginIcon(loginState: LoginState, modifier: Modifier = Modifier) {
    Box(contentAlignment = Alignment.Center, modifier = modifier.size(Dimens.bigIconSize)) {
        when (loginState) {
            is Idle ->
                if (loginState.loginError != null) {
                    Image(
                        painter = painterResource(id = R.drawable.icon_fail),
                        contentDescription = stringResource(id = R.string.login_fail_title),
                        contentScale = ContentScale.Inside,
                    )
                } else {
                    // If view is Idle, we display empty box to keep the same size as other states
                }
            is Loading -> MullvadCircularProgressIndicatorLarge()
            Success ->
                Image(
                    painter = painterResource(id = R.drawable.icon_success),
                    contentDescription = stringResource(id = R.string.logged_in_title),
                )
        }
    }
}

@Composable
private fun LoginState.title(): String =
    stringResource(
        id =
            when (this) {
                is Idle ->
                    when (this.loginError) {
                        is LoginError -> R.string.login_fail_title
                        null -> R.string.login_title
                    }
                is Loading -> R.string.logging_in_title
                Success -> R.string.logged_in_title
            }
    )

@Composable
private fun LoginState.supportingText(): String? {
    val res =
        when (this) {
            is Idle -> {
                when (loginError) {
                    LoginError.InvalidCredentials -> R.string.login_fail_description
                    LoginError.UnableToCreateAccount -> R.string.failed_to_create_account
                    LoginError.NoInternetConnection -> R.string.no_internet_connection
                    is LoginError.Unknown -> R.string.error_occurred
                    null -> return null
                }
            }
            is Loading.CreatingAccount -> R.string.creating_new_account
            is Loading.LoggingIn -> R.string.logging_in_description
            Success -> R.string.logged_in_description
        }
    return stringResource(id = res)
}

@Composable
private fun AccountDropDownItem(
    modifier: Modifier = Modifier,
    accountNumber: String,
    enabled: Boolean,
    onClick: () -> Unit,
    onDeleteClick: () -> Unit,
) {
    Row(
        modifier =
            modifier
                .clip(
                    MaterialTheme.shapes.medium.copy(
                        topStart = CornerSize(0f),
                        topEnd = CornerSize(0f),
                    )
                )
                .background(MaterialTheme.colorScheme.background)
                .height(IntrinsicSize.Min),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier =
                Modifier.clickable(enabled = enabled, onClick = onClick)
                    .fillMaxHeight()
                    .weight(1f)
                    .padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding),
            contentAlignment = Alignment.CenterStart,
        ) {
            Text(text = accountNumber, overflow = TextOverflow.Clip)
        }
        IconButton(enabled = enabled, onClick = onDeleteClick) {
            Icon(
                imageVector = Icons.Default.Clear,
                contentDescription = null,
                modifier = Modifier.size(Dimens.listIconSize),
            )
        }
    }
}

@Composable
private fun CreateAccountPanel(onCreateAccountClick: () -> Unit, isEnabled: Boolean) {
    Column(
        Modifier.fillMaxWidth()
            .background(MaterialTheme.colorScheme.background)
            .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenVerticalMargin)
    ) {
        Text(
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
            text = stringResource(id = R.string.dont_have_an_account),
            color = MaterialTheme.colorScheme.onBackground,
        )
        PrimaryButton(
            modifier = Modifier.fillMaxWidth(),
            text = stringResource(id = R.string.create_account),
            isEnabled = isEnabled,
            onClick = onCreateAccountClick,
        )
    }
}
