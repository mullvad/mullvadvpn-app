package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Button
import androidx.compose.material.MaterialTheme
import androidx.compose.material.contentColorFor
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.viewmodel.CellUiState

@Preview
@Composable
fun testUi() {

    Column(Modifier.background(MullvadDarkBlue)) {
        MtuComposeCell("", {}, {})

        Spacer(
            modifier = Modifier
                .fillMaxWidth()
                .height(8.dp)
        )

        NavigationComposeCell(title = stringResource(id = R.string.split_tunneling)) {
        }

        var list = ArrayList<String>()
        CustomDnsComposeCell(
            isEnabled = true,
            onToggle = {}
        )
    }
}

@Composable
fun BaseCell(
    onCellClicked: (() -> Unit)? = null,

    title: @Composable () -> Unit,
    titleModifier: Modifier = Modifier,

    bodyView: @Composable () -> Unit,
    bodyViewModifier: Modifier = Modifier,

    expandableContent: (@Composable () -> Unit)? = null,
    expandableContentModifier: Modifier = Modifier,

    subtitle: @Composable (() -> Unit)? = null,
    subtitleModifier: Modifier = Modifier,

    backgroundColor: Color = MaterialTheme.colors.surface,
    contentColor: Color = contentColorFor(backgroundColor),

    uiState: CellUiState?,
) {
    var cellHeight = dimensionResource(id = R.dimen.cell_height)
    var cellInnerSpacing = dimensionResource(id = R.dimen.cell_inner_spacing)
    var cellVerticalSpacing = dimensionResource(id = R.dimen.cell_label_vertical_padding)
    var cellHorizontalSpacing = dimensionResource(id = R.dimen.cell_left_padding)
//    var expanded by remember { mutableStateOf(true) }
//
//    val rotateState = animateFloatAsState(
//        targetValue = if (expanded) 180F else 0F,
//    )

    Column() {
        ConstraintLayout(
            modifier = Modifier
                .fillMaxWidth()
                .height(cellHeight)
                .background(colorResource(id = R.color.blue))
        ) {
            val (clickReceiver, contentContainer) = createRefs()

            // Click listener
            onCellClicked?.let {
                Button(
                    onClick = it,
                    modifier = Modifier
                        .constrainAs(clickReceiver) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            start.linkTo(parent.start)
                            end.linkTo(parent.end)
                        }
                        .fillMaxWidth()
                        .fillMaxHeight()
                ) {
                    Spacer(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(cellHeight)
                    )
                }
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.Start,
                modifier = Modifier
                    .padding(start = cellHorizontalSpacing, end = cellHorizontalSpacing)
                    .height(cellHeight)
                    .constrainAs(contentContainer) {
                        start.linkTo(parent.start)
                        end.linkTo(parent.end)
                        bottom.linkTo(parent.bottom)
                        top.linkTo(parent.top)
                    }
            ) {
                // Cell title
                if (uiState?.showWarning == true) {
                    Image(
                        painter = painterResource(id = R.drawable.icon_alert),
                        contentDescription = null,
                        modifier = Modifier
                            .padding(
                                end = cellInnerSpacing,
                            )
                            .wrapContentWidth()
                            .fillMaxHeight()
                    )
                }
                title()

                Spacer(modifier = Modifier.weight(1.0f))

                bodyView?.let {
                    Column(
                        modifier = bodyViewModifier
                            .wrapContentWidth()
                            .wrapContentHeight()
                    ) {
                        bodyView()
                    }
                }
            }
        }

        expandableContent?.let { ec ->
            ec()
        }

        // Cell subtitle
        subtitle?.let {
            Column(
                modifier = subtitleModifier
                    .background(MullvadDarkBlue)
                    .padding(
                        start = cellHorizontalSpacing,
                        top = 6.dp,
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
