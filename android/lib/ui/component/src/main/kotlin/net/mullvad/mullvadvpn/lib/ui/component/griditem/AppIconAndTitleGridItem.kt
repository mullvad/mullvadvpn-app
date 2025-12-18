package net.mullvad.mullvadvpn.lib.ui.component.griditem

import android.graphics.drawable.AdaptiveIconDrawable
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.core.graphics.drawable.toBitmap
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.util.applyIfNotNull

private val APP_ICON_SIZE = 60.dp

@Preview
@Composable
private fun PreviewAppIconAndTitleGridItem() {
    AppTheme {
        FlowRow(Modifier.background(MaterialTheme.colorScheme.surface)) {
            AppIconAndTitleGridItem(
                appTitle = "Obfuscation",
                appIcon = R.mipmap.ic_launcher_notes,
                onClick = {},
            )
            AppIconAndTitleGridItem(
                appTitle = "Obfuscation",
                appIcon = R.mipmap.ic_banner,
                onClick = {},
            )
        }
    }
}

@Composable
fun AppIconAndTitleGridItem(
    modifier: Modifier = Modifier,
    appTitle: String,
    appIcon: Int,
    appIconContentDescription: String? = null,
    onClick: (() -> Unit)? = null,
    testTag: String? = null,
) {
    val resources = LocalResources.current
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier =
            modifier
                .applyIfNotNull(testTag) { testTag(it) }
                .applyIfNotNull(onClick) { clickable(onClick = it) },
    ) {
        val adaptiveIconDrawable = resources.getDrawable(appIcon, null) as? AdaptiveIconDrawable
        if (adaptiveIconDrawable != null) {
            Icon(
                bitmap = adaptiveIconDrawable.toBitmap().asImageBitmap(),
                contentDescription = appIconContentDescription,
                modifier = Modifier.size(APP_ICON_SIZE),
                tint = Color.Unspecified,
            )
        } else
            (Icon(
                painter = painterResource(appIcon),
                contentDescription = appIconContentDescription,
                modifier = Modifier.size(APP_ICON_SIZE),
                tint = Color.Unspecified,
            ))
        Spacer(modifier = Modifier.height(Dimens.tinyPadding))
        Text(
            text = appTitle,
            style = MaterialTheme.typography.labelLarge,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
