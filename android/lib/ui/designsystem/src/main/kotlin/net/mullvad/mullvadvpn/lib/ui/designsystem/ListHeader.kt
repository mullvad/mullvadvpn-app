package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

private val LIST_HEADER_MIN_HEIGHT = 48.dp

@Composable
fun ListHeader(text: String, modifier: Modifier = Modifier) {
    ListHeader(content = { Text(text = text) }, modifier = modifier)
}

@Composable
fun ListHeader(
    content: @Composable () -> Unit,
    modifier: Modifier = Modifier,
    actions: @Composable (RowScope.() -> Unit)? = null,
) {
    ProvideContentColorTextStyle(
        MaterialTheme.colorScheme.onBackground,
        MaterialTheme.typography.labelLarge,
    ) {
        Row(
            modifier =
                Modifier.defaultMinSize(minHeight = LIST_HEADER_MIN_HEIGHT)
                    .height(IntrinsicSize.Min)
                    .then(modifier),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            content()
            HorizontalDivider(
                Modifier.weight(1f, true).padding(start = Dimens.smallPadding),
                color =
                    MaterialTheme.colorScheme.onBackground.copy(
                        alpha = RelayListHeaderTokens.RelayListHeaderDividerAlpha
                    ),
            )
            actions?.invoke(this)
        }
    }
}

object RelayListHeaderTokens {
    const val RelayListHeaderDividerAlpha = 0.2f
}

@Preview(backgroundColor = 0xFF192E45, showBackground = true)
@Composable
fun PreviewListHeader() {
    AppTheme {
        Column {
            ListHeader(content = { Text("Header") })
            ListHeader(
                content = { Text("Header") },
                actions = {
                    IconButton(onClick = {}) {
                        Icon(imageVector = Icons.Default.Edit, contentDescription = null)
                    }
                },
            )
        }
    }
}
