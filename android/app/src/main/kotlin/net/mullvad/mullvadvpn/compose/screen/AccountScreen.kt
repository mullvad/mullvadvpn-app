package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.component.CollapsableAwareToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.CopyableObfuscationView
import net.mullvad.mullvadvpn.compose.component.InformationView
import net.mullvad.mullvadvpn.compose.component.MissingPolicy
import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.ui.extension.openAccountPageInBrowser
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.util.toExpiryDateString
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewAccountScreen() {
    AccountScreen(
        uiState =
            AccountUiState(
                deviceName = "Test Name",
                accountNumber = "1234123412341234",
                accountExpiry = null
            ),
        viewAction = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
    )
}

@ExperimentalMaterial3Api
@Composable
fun AccountScreen(
    uiState: AccountUiState,
    viewAction: SharedFlow<AccountViewModel.ViewAction>,
    onRedeemVoucherClick: () -> Unit = {},
    onManageAccountClick: () -> Unit = {},
    onLogoutClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val context = LocalContext.current
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress

    CollapsableAwareToolbarScaffold(
        backgroundColor = MaterialTheme.colorScheme.background,
        modifier = Modifier.fillMaxSize(),
        state = state,
        scrollStrategy = ScrollStrategy.ExitUntilCollapsed,
        isEnabledWhenCollapsable = true,
        toolbar = {
            val scaffoldModifier =
                Modifier.road(
                    whenCollapsed = Alignment.TopCenter,
                    whenExpanded = Alignment.BottomStart
                )
            CollapsingTopBar(
                backgroundColor = MaterialTheme.colorScheme.secondary,
                onBackClicked = { onBackClick() },
                title = stringResource(id = R.string.settings_account),
                progress = progress,
                modifier = scaffoldModifier,
                backTitle = String(),
                shouldRotateBackButtonDown = true
            )
        },
    ) {
        LaunchedEffect(Unit) {
            viewAction.collect { viewAction ->
                if (viewAction is AccountViewModel.ViewAction.OpenAccountView) {
                    context.openAccountPageInBrowser(viewAction.token)
                }
            }
        }
        Column(
            verticalArrangement = Arrangement.Bottom,
            horizontalAlignment = Alignment.Start,
            modifier =
                Modifier.background(MaterialTheme.colorScheme.background)
                    .fillMaxWidth()
                    .wrapContentHeight()
                    .animateContentSize()
        ) {
            Text(
                style = MaterialTheme.typography.labelMedium,
                text = stringResource(id = R.string.device_name),
                modifier = Modifier.padding(start = Dimens.sideMargin, end = Dimens.sideMargin)
            )

            InformationView(
                content = uiState.deviceName.capitalizeFirstCharOfEachWord(),
                whenMissing = MissingPolicy.SHOW_SPINNER
            )

            Text(
                style = MaterialTheme.typography.labelMedium,
                text = stringResource(id = R.string.account_number),
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        top = Dimens.smallPadding
                    )
            )

            CopyableObfuscationView(content = uiState.accountNumber)

            Text(
                style = MaterialTheme.typography.labelMedium,
                text = stringResource(id = R.string.paid_until),
                modifier = Modifier.padding(start = Dimens.sideMargin, end = Dimens.sideMargin)
            )

            InformationView(
                content = uiState.accountExpiry?.toExpiryDateString() ?: "",
                whenMissing = MissingPolicy.SHOW_SPINNER
            )

            Spacer(modifier = Modifier.weight(1.0f))

            ActionButton(
                text = stringResource(id = R.string.manage_account),
                onClick = { onManageAccountClick() },
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    ),
                colors =
                    ButtonDefaults.buttonColors(
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                        containerColor = MaterialTheme.colorScheme.surface
                    )
            )

            ActionButton(
                text = stringResource(id = R.string.redeem_voucher),
                onClick = onRedeemVoucherClick,
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    ),
                colors =
                    ButtonDefaults.buttonColors(
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                        containerColor = MaterialTheme.colorScheme.surface
                    )
            )

            ActionButton(
                text = stringResource(id = R.string.log_out),
                onClick = onLogoutClick,
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    ),
                colors =
                    ButtonDefaults.buttonColors(
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                        containerColor = MaterialTheme.colorScheme.error
                    )
            )

            Spacer(modifier = Modifier.height(Dimens.cellHeight))
        }
    }
}
