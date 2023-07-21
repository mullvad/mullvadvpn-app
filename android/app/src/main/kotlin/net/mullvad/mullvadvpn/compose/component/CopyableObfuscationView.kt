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
import net.mullvad.mullvadvpn.ui.extension.copyToClipboard

@Preview
@Composable
private fun PreviewCopyableObfuscationView() {
    CopyableObfuscationView("1111222233334444")
}

@Composable
fun CopyableObfuscationView(content: String) {
    val context = LocalContext.current
    val shouldObfuscated = remember { mutableStateOf(true) }

    Row(verticalAlignment = Alignment.CenterVertically) {
        AccountNumberView(accountNumber = content, shouldObfuscated = shouldObfuscated.value)
        Spacer(modifier = Modifier.weight(1f))
        Image(
            painter =
                painterResource(
                    id = if (shouldObfuscated.value) R.drawable.icon_hide else R.drawable.icon_show
                ),
            modifier =
                Modifier.clickable { shouldObfuscated.value = shouldObfuscated.value.not() }
                    .padding(start = Dimens.sideMargin),
            contentDescription = stringResource(id = R.string.copy_account_number)
        )
        Image(
            painter = painterResource(id = R.drawable.icon_copy),
            modifier =
                Modifier.clickable {
                        context.copyToClipboard(
                            content = content,
                            clipboardLabel = context.getString(R.string.mullvad_account_number),
                            copiedToastMessage =
                                context.getString(R.string.copied_mullvad_account_number)
                        )
                    }
                    .padding(start = Dimens.sideMargin, end = Dimens.sideMargin),
            contentDescription = stringResource(id = R.string.copy_account_number)
        )
    }
}
