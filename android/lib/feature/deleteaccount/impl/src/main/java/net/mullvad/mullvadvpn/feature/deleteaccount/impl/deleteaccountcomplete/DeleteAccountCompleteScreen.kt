package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountcomplete

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Image
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.login.api.LoginNavKey
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha60

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Supported|Unsupported")
@Composable
private fun PreviewDeleteAccountComplete() {
    AppTheme { DeleteAccountComplete({}) }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeleteAccountComplete(navigator: Navigator) {
    val navigateToLogin = { navigator.navigate(LoginNavKey(), clearBackStack = true) }
    BackHandler(onBack = navigateToLogin)
    DeleteAccountComplete(onContinue = navigateToLogin)
}

@ExperimentalMaterial3Api
@Composable
fun DeleteAccountComplete(onContinue: () -> Unit) {
    ScaffoldWithSmallTopBar(
        appBarTitle = "",
        navigationIcon = { NavigateCloseIconButton(onContinue) },
    ) { modifier ->
        DeleteAccountCompleteContent(modifier, onContinue)
    }
}

const val DeleteWeightTop = 0.4f
const val DeleteWeightBottom = 0.6f

@Composable
private fun DeleteAccountCompleteContent(modifier: Modifier = Modifier, onContinue: () -> Unit) {
    Column(Modifier.padding(horizontal = Dimens.sideMarginNew)) {
        Column(
            modifier = modifier.fillMaxWidth().weight(1f),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Spacer(modifier = Modifier.weight(DeleteWeightTop))
            Image(
                painter = painterResource(id = R.drawable.icon_success),
                contentDescription = null,
            )
            Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
            Text(
                stringResource(R.string.account_deleted),
                style = MaterialTheme.typography.headlineSmall,
            )
            Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
            Text(
                modifier = Modifier.padding(horizontal = Dimens.mediumSpacer),
                text = stringResource(R.string.delete_account_complete_subtitle),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = Alpha60),
                textAlign = TextAlign.Center,
            )
            Spacer(modifier = Modifier.weight(DeleteWeightBottom))
        }

        PrimaryButton(
            modifier =
                Modifier.windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom))
                    .padding(
                        start = Dimens.smallPadding,
                        end = Dimens.smallPadding,
                        bottom = Dimens.screenBottomMargin,
                    ),
            onClick = onContinue,
            text = stringResource(R.string.go_to_login),
        )
    }
}
