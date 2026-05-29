package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha40

@Composable
fun Accordion(
    modifier: Modifier = Modifier,
    title: String,
    expandedText: AnnotatedString,
    isExpanded: Boolean,
    icon: ImageVector,
    iconContentDescription: String? = null,
    iconTint: Color = LocalContentColor.current,
    onClick: () -> Unit = {},
) {
    Column(
        modifier =
            modifier
                .background(
                    color = MaterialTheme.colorScheme.tertiaryContainer.copy(alpha = Alpha40),
                    shape = MaterialTheme.shapes.medium,
                )
                .clip(MaterialTheme.shapes.medium)
                .animateContentSize()
    ) {
        Row(
            modifier =
                Modifier.fillMaxWidth()
                    .clickable(onClick = onClick)
                    .padding(
                        top = Dimens.smallPadding,
                        start = Dimens.smallPadding,
                        end = Dimens.smallPadding,
                        bottom = if (isExpanded) Dimens.tinyPadding else Dimens.smallPadding,
                    )
        ) {
            Icon(imageVector = icon, contentDescription = iconContentDescription, tint = iconTint)
            Text(
                modifier =
                    Modifier.padding(horizontal = Dimens.smallPadding)
                        .weight(1f)
                        .align(Alignment.CenterVertically),
                style = MaterialTheme.typography.labelLarge,
                text = title,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            ExpandChevron(isExpanded = isExpanded)
        }
        if (isExpanded) {
            Text(
                modifier =
                    Modifier.padding(horizontal = Dimens.smallPadding)
                        .padding(bottom = Dimens.smallPadding),
                style = MaterialTheme.typography.bodyMedium,
                text = expandedText,
            )
        }
    }
}

@Preview
@Composable
private fun PreviewAccordion() {
    AppTheme {
        Surface {
            Column {
                Accordion(
                    isExpanded = false,
                    title = "This impacts your anonymity",
                    expandedText = "hello".toAnnotatedString(),
                    icon = Icons.Rounded.Info,
                )
                Spacer(Modifier.height(5.dp))
                Accordion(
                    isExpanded = true,
                    title = "How it works",
                    expandedText =
                        buildAnnotatedString {
                            appendLine(
                                annotatedStringResource(R.string.local_network_sharing_info3)
                            )
                            appendLine()
                            append(annotatedStringResource(R.string.local_network_sharing_info4))
                        },
                    icon = Icons.Rounded.Info,
                )
            }
        }
    }
}
