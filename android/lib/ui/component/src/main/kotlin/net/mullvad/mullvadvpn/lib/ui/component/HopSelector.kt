package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ErrorOutline
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.Language
import androidx.compose.material.icons.filled.PhoneAndroid
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material.icons.outlined.LocationOn
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.minimumInteractiveComponentSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight.Companion.SemiBold
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewFontScale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ExperimentalMotionApi
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.clip

enum class RelayList {
    Entry,
    Exit,
}

@PreviewFontScale
@Composable
fun MultiHopSelectorPreview() {
    AppTheme {
        Surface {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                //                MultiHop(entryLocation = "Sweden", exitLocation = "Germany")
                MultiHop(
                    exitSelected = false,
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                    exitFilters = 3,
                    entryFilters = 1,
                )
                MultiHop(
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                    entryFilters = 1,
                    entryErrorText = null,
                    exitErrorText = "No relays matching your selection",
                )

                MultiHop(
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                    entryFilters = 1,
                    exitErrorText = null,
                    entryErrorText = "No relays matching your selection",
                )
            }
        }
    }
}

@Composable
fun MultiHop(
    modifier: Modifier = Modifier,
    exitSelected: Boolean = true,
    entryLocation: String,
    entryErrorText: String? = null,
    entryFilters: Int = 0,
    onEntryFilterClick: () -> Unit = {},
    exitLocation: String,
    exitErrorText: String? = null,
    exitFilters: Int = 0,
    onExitFilterClick: () -> Unit = {},
    colors: HopSelectorColors = HopSelectorDefaults.colors(),
) {
    Column(modifier) {
        LocationHint("Internet", Icons.Default.Language, colors = colors)
        Column(
            modifier =
                Modifier.clip(RoundedCornerShape(16.dp))
                    .background(colors.panelColor)
                    .padding(horizontal = 4.dp)
        ) {
            Hop(
                modifier = Modifier,
                leadingIcon = Icons.Outlined.LocationOn,
                text = exitLocation,
                selected = exitSelected,
                onSelect = {},
                errorText = exitErrorText,
                filters = exitFilters,
                onFilterClick = onExitFilterClick,
                colors = colors,
            )
            Hop(
                modifier =
                    Modifier.let { if (entryErrorText != null) it.padding(bottom = 4.dp) else it },
                leadingIcon = Icons.Outlined.Dns,
                text = entryLocation,
                selected = !exitSelected,
                onSelect = {},
                errorText = entryErrorText,
                filters = entryFilters,
                onFilterClick = onEntryFilterClick,
                colors = colors,
            )
        }
        LocationHint("Your device", Icons.Default.PhoneAndroid, colors = colors)
    }
}

@Preview
@Composable
fun SingleHopSelectorPreview() {
    AppTheme {
        Surface {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                SingleHop("Sweden")

                SingleHop("Germany", filters = 2)

                SingleHop("Norway", errorText = "No relays matching your selection", filters = 2)
            }
        }
    }
}

@Composable
fun SingleHop(
    exitLocation: String,
    errorText: String? = null,
    filters: Int = 0,
    onFilterClick: () -> Unit = {},
) {
    Column {
        LocationHint("Internet", Icons.Default.Language)
        Hop(
            modifier = Modifier.padding(horizontal = 4.dp),
            leadingIcon = Icons.Outlined.LocationOn,
            text = exitLocation,
            selected = true,
            onSelect = {},
            errorText = errorText,
            filters = filters,
            onFilterClick = onFilterClick,
        )
        LocationHint("Your device", Icons.Default.PhoneAndroid)
    }
}

@Preview
@Composable
fun HopPreview() {
    AppTheme {
        Surface {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                var error by remember { mutableStateOf(false) }
                Switch(error, { error = it })
                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    errorText = null,
                    onFilterClick = {},
                    filters = 0,
                    modifier = Modifier.fillMaxWidth(),
                )

                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    errorText = "No relays matching your selection",
                    onFilterClick = {},
                    filters = 0,
                    modifier = Modifier.fillMaxWidth(),
                )

                Hop(
                    leadingIcon = Icons.Outlined.Dns,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    errorText = "No relays matching your selection",
                    onFilterClick = {},
                    filters = 0,
                    modifier = Modifier.fillMaxWidth(),
                )

                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = false,
                    onSelect = {},
                    errorText =
                        "No relays matching your selection, multiple lines error will looks like this.",
                    onFilterClick = {},
                    filters = 0,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}

@OptIn(ExperimentalMotionApi::class)
@Composable
private fun Hop(
    leadingIcon: ImageVector,
    text: String,
    selected: Boolean,
    onSelect: () -> Unit,
    errorText: String?,
    filters: Int,
    onFilterClick: () -> Unit,
    modifier: Modifier = Modifier,
    colors: HopSelectorColors = HopSelectorDefaults.colors(),
) {

    var parentPosition by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var iconPosition by remember { mutableStateOf<LayoutCoordinates?>(null) }
    CompositionLocalProvider(
        LocalContentColor provides
            if (selected) MaterialTheme.colorScheme.onPrimary else deselectedColor
    ) {
        Column(
            Modifier
                .onGloballyPositioned { parentPosition = it }
                .drawWithContent {
                    drawContent()
                    val realParentPosition = parentPosition ?: return@drawWithContent
                    val realIconPosition = iconPosition ?: return@drawWithContent

                    val position = realParentPosition.localPositionOf(realIconPosition)
                    val width: Dp = 1.dp

                    val x = 16.dp.toPx()
                    drawLine(
                        color = colors.legendColor,
                        start = Offset(x = x, y = 0f),
                        strokeWidth = width.toPx(),
                        end = Offset(x = x, y = position.y),
                        cap = StrokeCap.Round,
                    )
                    drawLine(
                        color = colors.legendColor,
                        start = Offset(x = x, y = position.y + realIconPosition.size.height),
                        strokeWidth = width.toPx(),
                        end = Offset(x = x, y = size.height),
                        cap = StrokeCap.Round,
                    )
                }.then(modifier)
        ) {
            Row {
                Row(
                    modifier =
                        Modifier.semantics {
                                role = Role.Switch
                                this.selected = selected
                            }
                            .weight(1f)
                            .minimumInteractiveComponentSize()
                            .defaultMinSize(minHeight = 40.dp)
                            .clip(RoundedCornerShape(12.dp))
                            .background(colors.containerColor(selected))
                            .clickable(onClick = onSelect)
                            .let {
                                if (errorText != null)
                                    it.border(1.dp, colors.errorColor, RoundedCornerShape(12.dp))
                                else it
                            },
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        modifier =
                            Modifier.onGloballyPositioned { iconPosition = it }
                                .padding(horizontal = 4.dp, vertical = 2.dp)
                                .size(24.dp),
                        imageVector =
                            if (errorText == null) leadingIcon else Icons.Default.ErrorOutline,
                        tint = colors.leadingIconColor(selected, errorText != null),
                        contentDescription = null,
                    )
                    Text(
                        modifier = Modifier.fillMaxWidth(),
                        text = text,
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = SemiBold,
                    )
                }

                CompositionLocalProvider(LocalContentColor provides colors.selectedContentColor) {
                    FilterButton(onClick = onFilterClick, filters = filters)
                }
            }
            AnimatedVisibility(errorText != null) {
                Text(
                    modifier = Modifier.padding(start = 32.dp),
                    text = errorText ?: "",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }
    }
}

@Composable
private fun LocationHint(
    text: String,
    imageVector: ImageVector,
    modifier: Modifier = Modifier,
    colors: HopSelectorColors = HopSelectorDefaults.colors(),
) {
    CompositionLocalProvider(LocalContentColor provides colors.legendColor) {
        Row(
            modifier.padding(start = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(
                modifier = Modifier.padding(3.dp).size(18.dp),
                imageVector = imageVector,
                contentDescription = null,
            )
            Text(modifier = Modifier.weight(1f), text = text)
        }
    }
}

@Preview
@Composable
fun FilterButtonPreview() {
    AppTheme {
        Column {
            FilterButton()
            FilterButton(filters = 3)
            FilterButton(filters = 10)
        }
    }
}

@Composable
fun FilterButton(modifier: Modifier = Modifier, filters: Int = 0, onClick: () -> Unit = {}) {
    IconButton(modifier = modifier, onClick = onClick) {
        BadgedBox(
            badge = {
                if (filters > 0) {
                    Badge(containerColor = Color.Transparent) { Text(filters.toString()) }
                }
            }
        ) {
            Icon(imageVector = Icons.Default.FilterList, contentDescription = null)
        }
    }
}

enum class BoxSize {
    Small,
    Large,
}

data class BoxState(val size: BoxSize, val error: Boolean)

// @OptIn(ExperimentalSharedTransitionApi::class)
// @Composable
// @Preview
// fun Test() {
//
//    Column {
//        var error by remember { mutableStateOf(false) }
//        val seekingState = remember { SeekableTransitionState(BoxState(BoxSize.Small, error)) }
//        val scope = rememberCoroutineScope()
//        Column {
//            Row {
//                Button(
//                    onClick = {
//                        scope.launch { seekingState.animateTo(BoxState(BoxSize.Small, error)) }
//                    },
//                    Modifier.wrapContentWidth().weight(1f),
//                ) {
//                    Text("Animate Small")
//                }
//                Button(
//                    onClick = {
//                        scope.launch { seekingState.seekTo(0f, BoxState(BoxSize.Large, error)) }
//                    },
//                    Modifier.wrapContentWidth().weight(1f),
//                ) {
//                    Text("Seek Large")
//                }
//                Button(
//                    onClick = {
//                        scope.launch { seekingState.animateTo(BoxState(BoxSize.Large, error)) }
//                    },
//                    Modifier.wrapContentWidth().weight(1f),
//                ) {
//                    Text("Animate Large")
//                }
//            }
//        }
//        Switch(
//            error,
//            {
//                error = it
//                scope.launch {
//                    seekingState.animateTo(BoxState(seekingState.currentState.size, it))
//                }
//            },
//        )
//        Slider(
//            value = seekingState.fraction,
//            modifier = Modifier.systemGestureExclusion().padding(20.dp),
//            onValueChange = { value -> scope.launch { seekingState.seekTo(fraction = value) } },
//            onValueChangeFinished = {
//                scope.launch {
//                    val targetState =
//                        if (seekingState.fraction < 0.5f)
//                            BoxState(BoxSize.Small, seekingState.currentState.error)
//                        else BoxState(BoxSize.Large, seekingState.currentState.error)
//                    seekingState.animateTo(targetState)
//                }
//            },
//        )
//        val transition = rememberTransition(seekingState)
//
//        SharedTransitionLayout {
//            transition.AnimatedContent(
//                transitionSpec = {
//                    fadeIn(tween(easing = LinearEasing)) togetherWith
//                        fadeOut(tween(easing = LinearEasing))
//                }
//            ) { state ->
//                val key = rememberSharedContentState(key = "image")
//                var b1 by remember { mutableStateOf<LayoutCoordinates?>(null) }
//                var b2 by remember { mutableStateOf<LayoutCoordinates?>(null) }
//                Box {
//                    Column {
//                        if (state.size == BoxSize.Small) {
//                            LocationHint("Internet", Icons.Default.Language)
//                        }
//                        Column {
//                            Hop(
//                                Modifier.sharedElement(
//                                        sharedContentState = key,
//                                        animatedVisibilityScope = this@AnimatedContent,
//                                    )
//                                    .padding(4.dp),
//                                hopState = HopState("Sweden", 0, if (error) "whoopsy" else null),
//                                Icons.Default.LocationOn,
//                                selected = true,
//                                onSelect = {},
//                                onFilterClick = {},
//                            )
//                        }
//                        if (state.size == BoxSize.Small) {
//                            LocationHint("Your Device", Icons.Default.PhoneAndroid)
//                        }
//                    }
//                }
//
//                with(LocalDensity.current) {
//                    val b1Bottom =
//                        b1?.let { it.positionInParent().y + it.size.height.toFloat() } ?: 0f
//                    val b2Top = b2?.positionInParent()?.y ?: 0f
//
//                    VerticalLine(
//                        modifier =
//                            Modifier.height((b2Top - b1Bottom).toDp())
//                                .offset(21.dp, b1Bottom.toDp())
//                    )
//
//                    VerticalLine(
//                        modifier =
//                            Modifier.height((b2Top - b1Bottom).toDp())
//                                .offset(21.dp, b1Bottom.toDp())
//                    )
//                }
//            }
//        }
//    }
// }

// Based of ListItem
@Immutable
class HopSelectorColors(
    val selectedContentColor: Color,
    val deselectedContentColor: Color,
    val selectedContainerColor: Color,
    val deselectedContainerColor: Color,
    val panelColor: Color,
    val errorColor: Color,
    val legendColor: Color,
) {
    @Stable
    internal fun containerColor(selected: Boolean): Color =
        if (selected) selectedContainerColor else deselectedContainerColor

    @Stable
    internal fun headlineColor(selected: Boolean): Color =
        when {
            selected -> selectedContentColor
            else -> deselectedContainerColor
        }

    @Stable
    internal fun leadingIconColor(selected: Boolean, error: Boolean): Color =
        when {
            error -> errorColor
            selected -> selectedContentColor
            else -> deselectedContentColor
        }
}

private val deselectedColor = Color(0xFFA3ABB5)

object HopSelectorDefaults {
    @Composable
    fun colors(
        selectedContentColor: Color = MaterialTheme.colorScheme.onPrimary,
        deselectedContentColor: Color = deselectedColor,
        selectedContainerColor: Color = MaterialTheme.colorScheme.surfaceContainerHighest,
        deselectedContainerColor: Color = Color.Transparent,
        panelColor: Color = MaterialTheme.colorScheme.surfaceContainer,
        errorColor: Color = MaterialTheme.colorScheme.error,
        legendColor: Color = deselectedColor,
    ): HopSelectorColors =
        HopSelectorColors(
            selectedContentColor,
            deselectedContentColor,
            selectedContainerColor,
            deselectedContainerColor,
            panelColor,
            errorColor,
            legendColor,
        )
}
