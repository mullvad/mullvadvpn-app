package net.mullvad.mullvadvpn.compose.component

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
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue

@Composable
fun BaseCell(
    title: @Composable () -> Unit,
    bodyView: @Composable () -> Unit,
    modifier: Modifier = Modifier,
    onCellClicked: () -> Unit = {},
    subtitle: @Composable (() -> Unit)? = null,
    subtitleModifier: Modifier = Modifier
) {
    val cellHeight = dimensionResource(id = R.dimen.cell_height)
    val cellVerticalSpacing = dimensionResource(id = R.dimen.cell_label_vertical_padding)
    val cellHorizontalSpacing = dimensionResource(id = R.dimen.cell_left_padding)
    val subtitleVerticalSpacing = dimensionResource(id = R.dimen.cell_footer_top_padding)

    Column {
        ConstraintLayout(
            modifier = Modifier
                .fillMaxWidth()
                .height(cellHeight)
                .background(colorResource(id = R.color.blue))
        ) {
            val (contentContainer) = createRefs()

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.Start,
                modifier = Modifier
                    .height(cellHeight)
                    .constrainAs(contentContainer) {
                        start.linkTo(parent.start)
                        end.linkTo(parent.end)
                        bottom.linkTo(parent.bottom)
                        top.linkTo(parent.top)
                    }
                    .clickable { onCellClicked.invoke() }
                    .padding(start = cellHorizontalSpacing, end = cellHorizontalSpacing)

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
        }

        // Cell subtitle
        if (subtitle != null) {
            Column(
                modifier = subtitleModifier
                    .background(MullvadDarkBlue)
                    .padding(
                        start = cellHorizontalSpacing,
                        top = subtitleVerticalSpacing,
                        end = cellHorizontalSpacing,
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
