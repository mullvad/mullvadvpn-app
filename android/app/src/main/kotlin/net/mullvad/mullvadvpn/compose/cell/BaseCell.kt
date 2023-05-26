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
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

@Preview
@Composable
fun PreviewBaseCell() {
    AppTheme {
        Column {
            BaseCell(
                title = {
                    BaseCellTitle(
                        title = "Header title",
                        style = MaterialTheme.typography.titleMedium
                    )
                }
            )
            Spacer(modifier = Modifier.height(1.dp))
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
    title: @Composable () -> Unit,
    bodyView: @Composable () -> Unit = {},
    isRowEnabled: Boolean = true,
    onCellClicked: () -> Unit = {},
    subtitle: @Composable (() -> Unit)? = null,
    subtitleModifier: Modifier = Modifier,
    background: Color = MullvadBlue,
    startPadding: Dp = dimensionResource(id = R.dimen.cell_left_padding),
    endPadding: Dp = dimensionResource(id = R.dimen.cell_right_padding)
) {
    val cellHeight = dimensionResource(id = R.dimen.cell_height)
    val cellVerticalSpacing = dimensionResource(id = R.dimen.cell_label_vertical_padding)
    val subtitleVerticalSpacing = dimensionResource(id = R.dimen.cell_footer_top_padding)

    Column(modifier = Modifier.fillMaxWidth().wrapContentHeight().background(background)) {
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
                    .height(cellHeight)
                    .fillMaxWidth()
                    .padding(start = startPadding, end = endPadding)
        ) {
            title()

            Spacer(modifier = Modifier.weight(1.0f))

            Column(modifier = modifier.wrapContentWidth().wrapContentHeight()) { bodyView() }
        }

        if (subtitle != null) {
            Row(
                modifier =
                    subtitleModifier
                        .background(MullvadDarkBlue)
                        .padding(
                            start = startPadding,
                            top = subtitleVerticalSpacing,
                            end = endPadding,
                            bottom = cellVerticalSpacing
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
fun CellSubtitle(content: String, modifier: Modifier = Modifier) {
    Text(
        text = content,
        style = MaterialTheme.typography.labelMedium,
        color = MullvadWhite60,
        modifier = modifier.padding(start = Dimens.sideMargin, top = Dimens.smallPadding)
    )
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
