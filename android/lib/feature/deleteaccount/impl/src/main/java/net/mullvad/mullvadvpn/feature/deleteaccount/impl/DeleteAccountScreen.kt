package net.mullvad.mullvadvpn.feature.deleteaccount.impl

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Error
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
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
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.generated.deleteaccount.destinations.DeleteAccountConfirmationDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.ui.component.CheckboxConfirmation
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Supported|Unsupported")
@Composable
private fun PreviewDeleteAccount() {
    AppTheme {
        DeleteAccount(
            Lc.Content(DeleteAccountUiState(30)),
            navigateToConfirmAccountDeletion = {},
            onBackClick = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination<ExternalModuleGraph>(style = SlideInFromRightTransition::class)
@Composable
fun DeleteAccount(navigator: DestinationsNavigator) {
    val vm = koinViewModel<DeleteAccountViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle()
    DeleteAccount(
        state.value,
        navigateToConfirmAccountDeletion =
            dropUnlessResumed { navigator.navigate(DeleteAccountConfirmationDestination) },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@ExperimentalMaterial3Api
@Composable
fun DeleteAccount(
    state: Lc<Unit, DeleteAccountUiState>,
    navigateToConfirmAccountDeletion: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.delete_account),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        bottomBar = { DeleteAccountBottomBar(onClickContinue = navigateToConfirmAccountDeletion) },
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMarginNew),
        ) {
            when (state) {
                is Lc.Content -> DeleteAccountContent(state.value.daysLeft)
                is Lc.Loading -> CircularProgressIndicator()
            }
        }
    }
}

@Composable
private fun DeleteAccountContent(daysLeft: Long) {
    Column(modifier = Modifier.padding(bottom = Dimens.smallPadding).animateContentSize()) {
        Text(
            text =
                buildAnnotatedString {
                    append(stringResource(id = R.string.delete_account_info))
                    val bulletItems =
                        listOf(
                                R.string.delete_account_first_item,
                                R.string.delete_account_second_item,
                                R.string.delete_account_third_item,
                                R.string.delete_account_forth_item,
                                R.string.delete_account_fifth_item,
                            )
                            .map { stringResource(it) }
                    withBulletList { bulletItems.forEach { withBulletListItem { append(it) } } }
                }
        )
        if (daysLeft > 0) {
            Spacer(Modifier.height(Dimens.mediumSpacer))
            DaysLostWarning(daysLeft)
        }

        Spacer(Modifier.height(Dimens.largeSpacer))

        CantBeUndoneText()
    }
}

@Composable
internal fun DaysLostWarning(daysLeft: Long) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(Dimens.tinyPadding),
    ) {
        Icon(
            imageVector = Icons.Rounded.Error,
            tint = MaterialTheme.colorScheme.error,
            contentDescription = null,
        )
        Text(stringResource(R.string.delete_account_days_left_warning, daysLeft))
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
            .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenBottomMargin),
        verticalArrangement = Arrangement.spacedBy(Dimens.mediumSpacer),
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
