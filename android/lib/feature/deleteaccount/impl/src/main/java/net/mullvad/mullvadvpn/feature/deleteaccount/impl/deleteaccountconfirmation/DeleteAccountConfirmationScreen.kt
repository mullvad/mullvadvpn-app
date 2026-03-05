package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Supported|Unsupported")
@Composable
private fun PreviewDeleteAccountConfirmation() {
    AppTheme {
        DeleteAccountConfirmation(hasConfirmedAccount = true, deleteAccount = {}, onBackClick = {})
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination<ExternalModuleGraph>(style = SlideInFromRightTransition::class)
@Composable
fun DeleteAccountConfirmation(navigator: DestinationsNavigator) {
    DeleteAccountConfirmation(
        hasConfirmedAccount = true,
        deleteAccount = {},
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@ExperimentalMaterial3Api
@Composable
fun DeleteAccountConfirmation(
    hasConfirmedAccount: Boolean,
    deleteAccount: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.delete_account),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        bottomBar = {
            DeleteAccountConfirmationBottomBar(
                hasConfirmedAccount,
                onClickDeleteAccount = deleteAccount,
                onClickCancel = onBackClick,
            )
        },
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMarginNew),
        ) {
            DeleteAccountConfirmationContent()
        }
    }
}

@Composable private fun DeleteAccountConfirmationContent() {}

@Composable
private fun DeleteAccountConfirmationBottomBar(
    hasConfirmedAccount: Boolean,
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
        )
        PrimaryButton(onClick = onClickCancel, text = stringResource(R.string.cancel))
    }
}
