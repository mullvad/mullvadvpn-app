package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SecureTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.flow.collectLatest
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.DeleteAccountError
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
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
        bottomBar = {
            DeleteAccountConfirmationBottomBar(
                state.contentOrNull()?.hasConfirmedAccount ?: false,
                state.contentOrNull()?.isLoading ?: false,
                onClickDeleteAccount = deleteAccount,
                onClickCancel = onBackClick,
            )
        },
    ) { modifier ->
        when (state) {
            is Lc.Content ->
                DeleteAccountConfirmationContent(modifier, state.value, onAccountInputChanged)
            is Lc.Loading -> CircularProgressIndicator()
        }
    }
}

@Composable
private fun DeleteAccountConfirmationContent(
    modifier: Modifier,
    state: DeleteAccountConfirmationUiState,
    onAccountInputChanged: (String) -> Unit,
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
        SecureTextField(
            state = textFieldState,
            isError = state.deleteAccountError != null,
            supportingText = state.deleteAccountError?.let { { Text(text = it.toErrorMessage()) } },
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
    Column(
        Modifier.windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom))
            .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenBottomMargin)
    ) {
        NegativeButton(
            text = stringResource(R.string.delete_account),
            onClick = onClickDeleteAccount,
            isEnabled = hasConfirmedAccount,
            isLoading = isLoading,
        )
        PrimaryButton(onClick = onClickCancel, text = stringResource(R.string.cancel))
    }
}
