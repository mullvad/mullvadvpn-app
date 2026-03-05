package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.autofill.ContentType
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.Clipboard
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalTextToolbar
import androidx.compose.ui.platform.NativeClipboard
import androidx.compose.ui.platform.TextToolbar
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentType
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest
import net.mullvad.mullvadvpn.common.compose.accountNumberKeyboardType
import net.mullvad.mullvadvpn.common.compose.accountNumberOutputTransformation
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.DeleteAccountError
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Supported|Unsupported")
@Composable
private fun PreviewDeleteAccountConfirmation() {
    AppTheme {
        DeleteAccountConfirmation(
            state = Lc.Content(DeleteAccountConfirmationUiState()),
            onAccountInputChanged = {},
            deleteAccount = {},
            onBackClick = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination<ExternalModuleGraph>(style = SlideInFromRightTransition::class)
@Composable
fun DeleteAccountConfirmation(navigator: DestinationsNavigator) {
    val vm = koinViewModel<DeleteAccountConfirmationViewModel>()
    val uiState = vm.uiState.collectAsStateWithLifecycle()
    DeleteAccountConfirmation(
        state = uiState.value,
        deleteAccount = vm::deleteAccount,
        onAccountInputChanged = vm::onAccountInputChanged,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@ExperimentalMaterial3Api
@Composable
fun DeleteAccountConfirmation(
    state: Lc<Unit, DeleteAccountConfirmationUiState>,
    onAccountInputChanged: (String) -> Unit,
    deleteAccount: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.delete_account),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier ->
        when (state) {
            is Lc.Content ->
                DeleteAccountConfirmationContent(
                    modifier,
                    state.value,
                    onAccountInputChanged,
                    deleteAccount,
                    onClickCancel = onBackClick,
                )
            is Lc.Loading -> CircularProgressIndicator()
        }
    }
}

@Composable
private fun DeleteAccountConfirmationContent(
    modifier: Modifier,
    state: DeleteAccountConfirmationUiState,
    onAccountInputChanged: (String) -> Unit,
    onClickDeleteAccount: () -> Unit,
    onClickCancel: () -> Unit,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMarginNew),
    ) {
        val textFieldState = rememberTextFieldState()
        LaunchedEffect(textFieldState) {
            snapshotFlow { textFieldState.text.toString() }
                .collectLatest { onAccountInputChanged(it) }
        }

        var showLastChar by remember { mutableStateOf(false) }

        LaunchedEffect(textFieldState.text) {
            showLastChar = true
            delay(3000)
            showLastChar = false
        }

        var showPassword by remember { mutableStateOf(false) }
        val localClipboard = LocalClipboard.current
        val clipboard = remember(localClipboard) { NoOpClipboardManager(localClipboard) }
        CompositionLocalProvider(LocalClipboard provides clipboard) {
            TextField(
                state = textFieldState,
                modifier =
                    // Fix for DPad navigation
                    Modifier.semantics { contentType = ContentType.Password }.fillMaxWidth(),
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
                keyboardOptions =
                    KeyboardOptions(
                        imeAction = ImeAction.Done,
                        keyboardType = KeyboardType.accountNumberKeyboardType(LocalContext.current),
                    ),
                outputTransformation =
                    accountNumberOutputTransformation(showPassword, if (showLastChar) 1 else 0),
                colors = mullvadWhiteTextFieldColors(),
                textStyle =
                    MaterialTheme.typography.bodyLarge.copy(textDirection = TextDirection.Ltr),
                isError = state.deleteAccountError != null,
            )
        }

        Spacer(Modifier.weight(1f))
        DeleteAccountConfirmationBottomBar(
            state.hasConfirmedAccount,
            state.isLoading,
            onClickDeleteAccount = onClickDeleteAccount,
            onClickCancel = onClickCancel,
        )
    }
}

@Composable
private fun DeleteAccountError.toErrorMessage(): String =
    when (this) {
        is DeleteAccountError.Unknown -> "Something went wrong: ${t.message}"
    }

@Composable
private fun DeleteAccountConfirmationBottomBar(
    hasConfirmedAccount: Boolean,
    isLoading: Boolean,
    onClickDeleteAccount: () -> Unit,
    onClickCancel: () -> Unit,
) {
    Column {
        NegativeButton(
            text = stringResource(R.string.delete_account),
            onClick = onClickDeleteAccount,
            isEnabled = hasConfirmedAccount,
            isLoading = isLoading,
        )
        PrimaryButton(onClick = onClickCancel, text = stringResource(R.string.cancel))
    }
}

@Composable
private fun DisableCutCopy(content: @Composable () -> Unit) {
    val currentToolbar = LocalTextToolbar.current
    val copyDisabledToolbar =
        remember(currentToolbar) {
            object : TextToolbar by currentToolbar {
                override fun showMenu(
                    rect: Rect,
                    onCopyRequested: (() -> Unit)?,
                    onPasteRequested: (() -> Unit)?,
                    onCutRequested: (() -> Unit)?,
                    onSelectAllRequested: (() -> Unit)?,
                    onAutofillRequested: (() -> Unit)?,
                ) {
                    currentToolbar.hide()
                }
            }
        }
    CompositionLocalProvider(LocalTextToolbar provides copyDisabledToolbar, content)
}

// Hack to disable pasting
class NoOpClipboardManager(private val clipboard: Clipboard) : Clipboard {

    override suspend fun getClipEntry(): ClipEntry? {
        return null
    }

    override suspend fun setClipEntry(clipEntry: ClipEntry?) {
        // Do nothing
    }

    override val nativeClipboard: NativeClipboard
        get() = clipboard.nativeClipboard
}
