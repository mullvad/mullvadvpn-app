package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.util.copyToClipboard

@Preview
@Composable
private fun PreviewCopyableInformationView() {
    CopyableInformationView("1111222233334444")
}

@Composable
fun CopyableInformationView(content: String) {
    val context = LocalContext.current
    val isShown = remember { mutableStateOf(false) }
    val clipboardLabel = stringResource(id = R.string.mullvad_account_number)
    val copiedToastMessage = stringResource(id = R.string.copied_mullvad_account_number)

    return Row(verticalAlignment = Alignment.CenterVertically) {
        AccountNumberView(accountNumber = content, isShown = isShown.value)
        Spacer(modifier = Modifier.weight(1f))
        Image(
            painter =
                painterResource(
                    id = if (isShown.value) R.drawable.icon_hide else R.drawable.icon_show
                ),
            modifier =
                Modifier.clickable { isShown.value = isShown.value.not() }
                    .padding(start = Dimens.sideMargin),
            contentDescription = stringResource(id = R.string.copy_account_number)
        )
        Image(
            painter = painterResource(id = R.drawable.icon_copy),
            modifier =
                Modifier.clickable {
                        content.copyToClipboard(
                            context = context,
                            clipboardLabel = clipboardLabel,
                            copiedToastMessage = copiedToastMessage
                        )
                    }
                    .padding(start = Dimens.sideMargin, end = Dimens.sideMargin),
            contentDescription = stringResource(id = R.string.copy_account_number)
        )
    }
}
