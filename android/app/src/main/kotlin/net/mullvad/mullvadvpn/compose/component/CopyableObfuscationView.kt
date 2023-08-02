package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment.Companion.CenterVertically
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.AnimatedIconButton
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils
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

    Row(verticalAlignment = CenterVertically, horizontalArrangement = Arrangement.End) {
        AccountNumberView(
            accountNumber = content,
            doObfuscateWithPasswordDots = shouldObfuscated.value
        )
        Spacer(modifier = Modifier.weight(1f))
        AnimatedIconButton(
            defaultIcon = painterResource(id = R.drawable.icon_hide),
            secondaryIcon = painterResource(id = R.drawable.icon_show),
            isToggleButton = true,
            modifier = Modifier.padding(start = Dimens.smallPadding, end = Dimens.sideMargin),
            onClick = { shouldObfuscated.value = shouldObfuscated.value.not() }
        )
        AnimatedIconButton(
            defaultIcon = painterResource(id = R.drawable.icon_copy),
            secondaryIcon = painterResource(id = R.drawable.icon_tick),
            secondaryIconColorFilter =
                ColorFilter.tint(color = MaterialTheme.colorScheme.inversePrimary),
            isToggleButton = false,
            modifier = Modifier.padding(end = Dimens.sideMargin),
            onClick = {
                context.copyToClipboard(
                    content = content,
                    clipboardLabel = context.getString(R.string.mullvad_account_number)
                )
                SdkUtils.showCopyToastIfNeeded(
                    context,
                    context.getString(R.string.copied_mullvad_account_number)
                )
            }
        )
    }
}
