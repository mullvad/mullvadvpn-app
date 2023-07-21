package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.component.CopyableInformationView
import net.mullvad.mullvadvpn.compose.component.InformationView
import net.mullvad.mullvadvpn.compose.component.MissingPolicy
import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewAccountScreen() {
    AccountScreen(
        uiState =
            AccountUiState(
                deviceName = "Test Name",
                accountNumber = "1234123412341234",
                showAccountNumber = false,
                accountExpiry = null
            )
    )
}

@ExperimentalMaterial3Api
@Composable
fun AccountScreen(
    uiState: AccountUiState,
    onManageAccountClick: (Context) -> Unit = {},
    onRedeemVoucherClick: () -> Unit = {},
    onLogoutClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val context = LocalContext.current
    val iconSize = Dimens.listIconSize

    val systemUiController = rememberSystemUiController()
    systemUiController.setStatusBarColor(MaterialTheme.colorScheme.background)
    systemUiController.setNavigationBarColor(MaterialTheme.colorScheme.background)

    Column(
        verticalArrangement = Arrangement.Bottom,
        horizontalAlignment = Alignment.Start,
        modifier =
            Modifier.background(MaterialTheme.colorScheme.background)
                .fillMaxWidth()
                .fillMaxHeight()
                .animateContentSize()
    ) {
        Button(
            modifier = Modifier.wrapContentWidth().wrapContentHeight(),
            onClick = onBackClick,
            colors =
                ButtonDefaults.buttonColors(
                    contentColor = Color.White,
                    containerColor = MullvadDarkBlue
                ),
            shape = MaterialTheme.shapes.small
        ) {
            Image(
                painter = painterResource(id = R.drawable.icon_back),
                contentDescription = stringResource(id = R.string.back),
                modifier = Modifier.rotate(270f).width(iconSize).height(iconSize)
            )
        }

        Text(
            text = stringResource(id = R.string.settings_account),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.mediumPadding
                )
        )

        Text(
            style = MaterialTheme.typography.labelMedium,
            text = stringResource(id = R.string.device_name),
            modifier = Modifier.padding(start = Dimens.sideMargin, end = Dimens.sideMargin)
        )

        InformationView(content = uiState.deviceName, whenMissing = MissingPolicy.SHOW_SPINNER)

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

        CopyableInformationView(content = uiState.accountNumber)

        Text(
            style = MaterialTheme.typography.labelMedium,
            text = stringResource(id = R.string.paid_until),
            modifier = Modifier.padding(start = Dimens.sideMargin, end = Dimens.sideMargin)
        )

        InformationView(content = uiState.expiryString, whenMissing = MissingPolicy.SHOW_SPINNER)

        Spacer(modifier = Modifier.weight(1.0f))

        ActionButton(
            text = stringResource(id = R.string.manage_account),
            onClick = { onManageAccountClick(context) },
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
    }
}
