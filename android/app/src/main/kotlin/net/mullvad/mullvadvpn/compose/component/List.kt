package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R

@Composable
fun DeviceRow(
    name: String,
    painter: Painter? = null,
    onItemClicked: () -> Unit
) {
    val itemColor = colorResource(id = R.color.blue)

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 1.dp)
            .height(50.dp)
            .background(itemColor)
            .clickable {
                onItemClicked()
            },
    ) {
        Text(
            text = name,
            fontSize = 18.sp,
            color = Color.White,
            modifier = Modifier
                .padding(
                    horizontal = 16.dp
                )
                .align(Alignment.CenterStart)
        )

        if (painter != null) {
            Image(
                painter = painter,
                contentDescription = "Remove",
                modifier = Modifier
                    .align(Alignment.CenterEnd)
                    .padding(horizontal = 12.dp)
            )
        }
    }
}

@Composable
fun <T> ItemList(
    items: List<T>,
    itemText: (T) -> String,
    onItemClicked: (T) -> Unit,
    itemPainter: Painter? = null,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
    ) {
        items.forEach { item ->
            DeviceRow(itemText.invoke(item), itemPainter) {
                onItemClicked(item)
            }
        }
    }
}
