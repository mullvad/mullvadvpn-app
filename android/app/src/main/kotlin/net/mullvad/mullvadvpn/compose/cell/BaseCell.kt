package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Preview
@Composable
private fun PreviewBaseCell() {
    AppTheme {
        SpacedColumn {
            BaseCell(
                title = {
                    BaseCellTitle(
                        title = "Header title",
                        style = MaterialTheme.typography.titleMedium
                    )
                }
            )
            BaseCell(
                title = {
                    BaseCellTitle(
                        title = "Normal title",
                        style = MaterialTheme.typography.labelLarge
                    )
                }
            )
        }
    }
}

@Composable
internal fun BaseCell(
    modifier: Modifier = Modifier,
    iconView: @Composable () -> Unit = {},
    title: @Composable () -> Unit,
    bodyView: @Composable () -> Unit = {},
    isRowEnabled: Boolean = true,
    onCellClicked: () -> Unit = {},
    subtitle: @Composable (() -> Unit)? = null,
    subtitleModifier: Modifier = Modifier,
    background: Color = MaterialTheme.colorScheme.primary,
    startPadding: Dp = Dimens.cellStartPadding,
    endPadding: Dp = Dimens.cellEndPadding,
    testTag: String = ""
) {
    Column(
        modifier =
            Modifier.fillMaxWidth().wrapContentHeight().background(background).testTag(testTag)
    ) {
        val rowModifier =
            Modifier.let {
                if (isRowEnabled) {
                    it.clickable { onCellClicked() }
                } else it
            }
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Start,
            modifier =
                rowModifier
                    .height(Dimens.cellHeight)
                    .fillMaxWidth()
                    .padding(start = startPadding, end = endPadding)
        ) {
            iconView()

            title()

            Spacer(modifier = Modifier.weight(1.0f))

            Column(modifier = modifier.wrapContentWidth().wrapContentHeight()) { bodyView() }
        }

        if (subtitle != null) {
            Row(
                modifier =
                    subtitleModifier
                        .background(MaterialTheme.colorScheme.secondary)
                        .padding(
                            start = startPadding,
                            top = Dimens.cellFooterTopPadding,
                            end = endPadding,
                            bottom = Dimens.cellLabelVerticalPadding
                        )
                        .fillMaxWidth()
                        .wrapContentHeight()
            ) {
                subtitle()
            }
        }
    }
}

@Composable
internal fun BaseCellTitle(title: String, style: TextStyle, modifier: Modifier = Modifier) {
    Text(
        text = title,
        textAlign = TextAlign.Center,
        style = style,
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = modifier.wrapContentWidth(align = Alignment.End).wrapContentHeight()
    )
}
