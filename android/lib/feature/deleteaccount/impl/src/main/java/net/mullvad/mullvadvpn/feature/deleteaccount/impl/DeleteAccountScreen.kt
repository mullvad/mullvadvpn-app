package net.mullvad.mullvadvpn.feature.deleteaccount.impl

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.generated.deleteaccount.destinations.DeleteAccountConfirmationDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.ui.component.CheckboxConfirmation
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Supported|Unsupported")
@Composable
private fun PreviewDeleteAccount() {
    AppTheme { DeleteAccount(navigateToConfirmAccountDeletion = {}, onBackClick = {}) }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination<ExternalModuleGraph>(style = SlideInFromRightTransition::class)
@Composable
fun DeleteAccount(navigator: DestinationsNavigator) {
    DeleteAccount(
        navigateToConfirmAccountDeletion =
            dropUnlessResumed { navigator.navigate(DeleteAccountConfirmationDestination) },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@ExperimentalMaterial3Api
@Composable
fun DeleteAccount(navigateToConfirmAccountDeletion: () -> Unit, onBackClick: () -> Unit) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.delete_account),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        bottomBar = { DeleteAccountBottomBar(onClickContinue = navigateToConfirmAccountDeletion) },
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMarginNew),
        ) {
            DeleteAccountContent()
        }
    }
}

@Composable
private fun DeleteAccountContent() {
    Column(modifier = Modifier.padding(bottom = Dimens.smallPadding).animateContentSize()) {
        Text(
            text =
                "You are about to delete your Mullvad VPN account. By deleting your account:\n" +
                    "All current devices will be logged out.\n" +
                    "The account number will become invalid.\n" +
                    "Any time left will be lost."
        )
    }
}

@Composable
private fun DeleteAccountBottomBar(onClickContinue: () -> Unit) {
    Column(
        Modifier.windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom))
            .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenBottomMargin)
    ) {
        var confirmed by rememberSaveable { mutableStateOf(false) }
        CheckboxConfirmation(
            text = stringResource(R.string.delete_account_confirmation_check),
            checked = confirmed,
            onCheckedChange = { confirmed = it },
        )
        PrimaryButton(
            onClick = onClickContinue,
            text = stringResource(R.string.cont),
            isEnabled = confirmed,
        )
    }
}
