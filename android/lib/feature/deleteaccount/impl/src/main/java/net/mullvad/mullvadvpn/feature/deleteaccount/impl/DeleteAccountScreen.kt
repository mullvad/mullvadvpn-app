package net.mullvad.mullvadvpn.feature.deleteaccount.impl

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.deleteaccount.api.DeleteAccountConfirmationNavKey
import net.mullvad.mullvadvpn.lib.ui.component.CheckboxConfirmation
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
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
@Composable
fun DeleteAccount(navigator: Navigator) {
    DeleteAccount(
        navigateToConfirmAccountDeletion =
            dropUnlessResumed { navigator.navigate(DeleteAccountConfirmationNavKey) },
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@ExperimentalMaterial3Api
@Composable
fun DeleteAccount(navigateToConfirmAccountDeletion: () -> Unit, onBackClick: () -> Unit) {
    ScaffoldWithSmallTopBar(
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
        Text(stringResource(id = R.string.delete_account_info))
        Spacer(modifier = Modifier.height(Dimens.tinyPadding))
        Text(stringResource(id = R.string.delete_account_info_part2))
        Text(
            text =
                buildAnnotatedString {
                    val bulletItems =
                        listOf(
                                R.string.delete_account_first_item,
                                R.string.delete_account_second_item,
                                R.string.delete_account_third_item,
                                R.string.delete_account_forth_item,
                            )
                            .map { annotatedStringResource(it) }
                    withBulletList { bulletItems.forEach { withBulletListItem { append(it) } } }
                },
            style = MaterialTheme.typography.bodyLarge,
        )

        Spacer(Modifier.height(Dimens.largeSpacer))

        CantBeUndoneText()
    }
}

@Composable
internal fun CantBeUndoneText() {
    Text(
        modifier = Modifier.fillMaxWidth(),
        text = stringResource(R.string.delete_account_cant_be_undone),
        style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
    )
}

@Composable
private fun DeleteAccountBottomBar(onClickContinue: () -> Unit) {
    Column(
        Modifier.windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom))
            .padding(horizontal = Dimens.sideMarginNew, vertical = Dimens.screenBottomMargin),
        verticalArrangement = Arrangement.spacedBy(Dimens.mediumSpacer),
    ) {
        var confirmed by rememberSaveable { mutableStateOf(false) }
        CheckboxConfirmation(
            text = stringResource(R.string.delete_account_confirmation_check),
            checked = confirmed,
            onCheckedChange = { confirmed = it },
        )
        PrimaryButton(
            modifier = Modifier.padding(horizontal = Dimens.smallPadding),
            onClick = {
                onClickContinue()
                // Clear it so they have to check it again if navigating back
                confirmed = false
            },
            text = stringResource(R.string.cont),
            isEnabled = confirmed,
        )
    }
}
