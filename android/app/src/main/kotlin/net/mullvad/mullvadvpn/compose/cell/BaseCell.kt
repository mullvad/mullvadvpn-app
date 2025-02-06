package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewBaseCell() {
    AppTheme {
        SpacedColumn(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
            BaseCell(
                headlineContent = {
                    BaseCellTitle(
                        title = "Header title",
                        style = MaterialTheme.typography.titleMedium,
                    )
                }
            )
            BaseCell(
                headlineContent = {
                    BaseCellTitle(
                        title = "Normal title",
                        style = MaterialTheme.typography.labelLarge,
                    )
                }
            )
        }
    }
}

@Composable
internal fun BaseCell(
    modifier: Modifier = Modifier,
    iconView: @Composable RowScope.() -> Unit = {},
    headlineContent: @Composable RowScope.() -> Unit,
    bodyView: @Composable ColumnScope.() -> Unit = {},
    isRowEnabled: Boolean = true,
    onCellClicked: (() -> Unit)? = null,
    background: Color = MaterialTheme.colorScheme.primary,
    startPadding: Dp = Dimens.cellStartPadding,
    endPadding: Dp = Dimens.cellEndPadding,
    minHeight: Dp = Dimens.cellHeight,
    testTag: String = "",
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Start,
        modifier =
            modifier
                // This is to avoid a crash when a child view is focused and clickable is set to
                // false on the parent view
                .then(
                    if (isRowEnabled && onCellClicked != null) {
                        Modifier.clickable(onClick = onCellClicked)
                    } else {
                        Modifier
                    }
                )
                .wrapContentHeight()
                .defaultMinSize(minHeight = minHeight)
                .fillMaxWidth()
                .background(background)
                .testTag(testTag)
                .padding(start = startPadding, end = endPadding),
    ) {
        iconView()

        headlineContent()

        Column(modifier = Modifier.wrapContentWidth().wrapContentHeight()) { bodyView() }
    }
}

@Composable
internal fun BaseCellTitle(
    title: String,
    style: TextStyle,
    modifier: Modifier = Modifier,
    textAlign: TextAlign = TextAlign.Start,
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
) {
    Text(
        text = title,
        textAlign = textAlign,
        style = style,
        color = textColor,
        overflow = TextOverflow.Ellipsis,
        maxLines = 1,
        modifier = modifier,
    )
}

@Composable
fun BaseSubtitleCell(
    text: String,
    modifier: Modifier = Modifier,
    style: TextStyle = MaterialTheme.typography.labelMedium,
    color: Color,
) {
    BaseSubtitleCell(
        text = AnnotatedString(text),
        modifier = modifier,
        style = style,
        color = color,
    )
}

@Composable
fun BaseSubtitleCell(
    text: AnnotatedString,
    modifier: Modifier = Modifier,
    style: TextStyle = MaterialTheme.typography.labelMedium,
    color: Color,
) {
    Text(
        text = text,
        style = style,
        color = color,
        modifier =
            modifier
                .padding(
                    start = Dimens.cellStartPadding,
                    top = Dimens.cellFooterTopPadding,
                    end = Dimens.cellEndPadding,
                    bottom = Dimens.cellVerticalSpacing,
                )
                .fillMaxWidth()
                .wrapContentHeight(),
    )
}
