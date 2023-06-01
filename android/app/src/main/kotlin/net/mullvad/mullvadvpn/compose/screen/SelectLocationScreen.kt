package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.RelayLocation
import net.mullvad.mullvadvpn.compose.component.SearchTextField
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem

@Preview
@Composable
fun PreviewSelectLocationScreen() {
    val state =
        SelectLocationUiState.Data.Show(
            countries = listOf(RelayCountry("Country 1", "Code 1", false, emptyList())),
            selectedRelay = null
        )
    AppTheme { SelectLocationScreen(uiState = state) }
}

@Composable
fun SelectLocationScreen(
    uiState: SelectLocationUiState,
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    if (uiState is SelectLocationUiState.Data.Close) {
        LaunchedEffect(uiState) { onBackClick() }
    }
    LazyColumn(modifier = Modifier.background(MaterialTheme.colorScheme.background)) {
        item(contentType = ContentType.TITLE) {
            Row(modifier = Modifier.padding(horizontal = 12.dp, vertical = 12.dp).fillMaxWidth()) {
                Image(
                    painter = painterResource(id = R.drawable.icon_back),
                    contentDescription = null,
                    modifier = Modifier.size(24.dp).rotate(90f)
                )
                Text(
                    text = stringResource(id = R.string.select_location),
                    modifier = Modifier.align(Alignment.CenterVertically).weight(weight = 1f),
                    textAlign = TextAlign.Center,
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onPrimary
                )
                Image(
                    painter = painterResource(id = R.drawable.icons_more_circle),
                    contentDescription = null,
                    modifier = Modifier.size(24.dp),
                    colorFilter = ColorFilter.tint(color = MaterialTheme.colorScheme.onSecondary)
                )
            }
        }
        item( contentType = 99) {
            SearchTextField {
                //On change
            }
        }
        item(contentType = ContentType.SPACER) {
            Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))
        }
        when (uiState) {
            SelectLocationUiState.Loading -> {
                item(contentType = ContentType.PROGRESS) {
                    CircularProgressIndicator(
                        color = MaterialTheme.colorScheme.onBackground,
                        modifier =
                            Modifier.size(
                                width = Dimens.progressIndicatorSize,
                                height = Dimens.progressIndicatorSize
                            )
                    )
                }
            }
            is SelectLocationUiState.Data -> {
                items(
                    count = uiState.countries.size,
                    key = { index -> uiState.countries[index].code },
                    contentType = { ContentType.ITEM }
                ) { index ->
                    val country = uiState.countries[index]
                    RelayLocation(
                        relay = country,
                        selectedItem = uiState.selectedRelay,
                        onSelectRelay = onSelectRelay,
                        modifier = Modifier.animateContentSize()
                    )
                }
            }
        }
    }
}
