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
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue

@Composable
fun BaseCell(
    title: @Composable () -> Unit,
    bodyView: @Composable () -> Unit,
    modifier: Modifier = Modifier,
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

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .wrapContentHeight()
            .background(background)
    ) {

        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Start,
            modifier = Modifier
                .height(cellHeight)
                .fillMaxWidth()
                .clickable { onCellClicked.invoke() }
                .padding(start = startPadding, end = endPadding)

        ) {
            title()

            Spacer(modifier = Modifier.weight(1.0f))

            Column(
                modifier = modifier
                    .wrapContentWidth()
                    .wrapContentHeight()
            ) {
                bodyView()
            }
        }

        if (subtitle != null) {
            Row(
                modifier = subtitleModifier
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
