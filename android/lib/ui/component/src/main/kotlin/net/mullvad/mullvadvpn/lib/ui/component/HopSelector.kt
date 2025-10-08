package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
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
import androidx.compose.ui.graphics.drawscope.DrawScope
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
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.Dimension
import androidx.constraintlayout.compose.ExperimentalMotionApi
import androidx.constraintlayout.compose.MotionLayout
import androidx.constraintlayout.compose.MotionScene
import androidx.constraintlayout.compose.Visibility
import androidx.constraintlayout.compose.layoutId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

enum class RelayList {
    Entry,
    Exit,
}

private const val keyInternet = "internet"
private const val keyPanel = "panel"
private const val keyExit = "exit"
private const val keyExitError = "exitError"
private const val keyEntry = "entry"
private const val keyEntryError = "entryError"
private const val keyDevice = "device"

@OptIn(ExperimentalSharedTransitionApi::class)
@Preview
@Composable
private fun CollapsibleSinglehopSelector() {

    var progress by remember { mutableFloatStateOf(1f) }
    var isExitError by remember { mutableStateOf(false) }

    AppTheme {
        Column(Modifier.background(MaterialTheme.colorScheme.background).padding(16.dp)) {
            Slider(
                value = progress,
                modifier = Modifier.padding(20.dp),
                onValueChange = { value -> progress = value },
                onValueChangeFinished = {},
            )

            Switch(isExitError, { isExitError = it })

            Singlehop(
                expandProgress = progress,
                errorText = if (isExitError) "Derp herp" else null,
                exitLocation = "Gothenburg",
            )
        }
    }
}

@Preview
@Composable
private fun CollapsibleMultihopSelector() {
    var progress by remember { mutableFloatStateOf(1f) }
    var exitSelected by remember { mutableStateOf(true) }
    var isExitError by remember { mutableStateOf(false) }
    var isEntryError by remember { mutableStateOf(false) }

    AppTheme {
        Column(Modifier.background(MaterialTheme.colorScheme.background).padding(16.dp)) {
            Slider(
                value = progress,
                modifier = Modifier.padding(20.dp),
                onValueChange = { value -> progress = value },
                onValueChangeFinished = {},
            )

            Switch(isExitError, onCheckedChange = { isExitError = it })
            Switch(isEntryError, onCheckedChange = { isEntryError = it })

            MultihopSelector(
                modifier = Modifier,
                exitSelected,
                "Germany",
                if (isEntryError) "No relays matching your selection" else null,
                { exitSelected = false },
                2,
                {},
                "Sweden",
                if (isExitError) "No relays matching your selection" else null,
                { exitSelected = true },
                0,
                {},
                expandProgress = progress,
            )
        }
    }
}

@OptIn(ExperimentalSharedTransitionApi::class)
@PreviewFontScale
@Preview
@Composable
fun MultihopSelectorPreview() {
    AppTheme {
        Surface {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                MultihopSelector(
                    exitSelected = false,
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                    exitFilters = 3,
                    entryFilters = 1,
                )
                MultihopSelector(
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                    entryFilters = 1,
                    entryErrorText = null,
                    exitErrorText = "No relays matching your selection",
                )

                MultihopSelector(
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

@OptIn(ExperimentalSharedTransitionApi::class, ExperimentalMotionApi::class)
@Composable
fun MultihopSelector(
    modifier: Modifier = Modifier,
    exitSelected: Boolean = true,
    entryLocation: String,
    entryErrorText: String? = null,
    onEntryClick: () -> Unit = {},
    entryFilters: Int = 0,
    onEntryFilterClick: () -> Unit = {},
    exitLocation: String,
    exitErrorText: String? = null,
    onExitClick: () -> Unit = {},
    exitFilters: Int = 0,
    onExitFilterClick: () -> Unit = {},
    expandProgress: Float = 1f,
) {
    val scene = MotionScene {
        val expandSet =
            constraintSet("expanded") {
                val (internet, exit, exitError, entry, entryError, device, panel) =
                    createRefsFor(
                        keyInternet,
                        keyExit,
                        keyExitError,
                        keyEntry,
                        keyEntryError,
                        keyDevice,
                        keyPanel,
                    )
                createVerticalChain(internet, exit, exitError, entry, entryError, device)
                constrain(exit) { linkTo(start = parent.start, end = parent.end) }
                constrain(entry) { linkTo(start = parent.start, end = parent.end) }
                constrain(exitError) {
                    visibility = if (exitErrorText == null) Visibility.Gone else Visibility.Visible
                    linkTo(
                        start = parent.start,
                        end = parent.end,
                        startMargin = 28.dp,
                        endMargin = 8.dp,
                    )
                }
                constrain(entryError) {
                    visibility = if (entryErrorText == null) Visibility.Gone else Visibility.Visible
                    linkTo(
                        start = parent.start,
                        end = parent.end,
                        startMargin = 28.dp,
                        endMargin = 8.dp,
                    )
                }
                constrain(panel) {
                    width = Dimension.fillToConstraints
                    height = Dimension.fillToConstraints
                    linkTo(
                        top = exit.top,
                        bottom = entryError.bottom,
                        start = exit.start,
                        end = exit.end,
                    )
                }
            }

        val collapseSet =
            constraintSet("collapsed") {
                val (internet, exit, exitError, entry, entryError, device, panel) =
                    createRefsFor(
                        keyInternet,
                        keyExit,
                        keyExitError,
                        keyEntry,
                        keyEntryError,
                        keyDevice,
                        keyPanel,
                    )

                constrain(internet) { top.linkTo(parent.top) }
                constrain(device) { bottom.linkTo(parent.bottom) }

                createVerticalChain(exit, exitError, entry, entryError)

                constrain(exit) { linkTo(start = parent.start, end = parent.end) }
                constrain(entry) { linkTo(start = parent.start, end = parent.end) }

                constrain(exitError) {
                    visibility = if (exitErrorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(
                        start = parent.start,
                        end = parent.end,
                        startMargin = 28.dp,
                        endMargin = 8.dp,
                    )
                }
                constrain(entryError) {
                    visibility = if (entryErrorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(
                        start = parent.start,
                        end = parent.end,
                        startMargin = 28.dp,
                        endMargin = 8.dp,
                    )
                }

                constrain(panel) {
                    width = Dimension.fillToConstraints
                    height = Dimension.fillToConstraints
                    linkTo(
                        top = exit.top,
                        bottom = entryError.bottom,
                        start = exit.start,
                        end = exit.end,
                    )
                }
            }

        defaultTransition(collapseSet, expandSet) {}
    }

    var motionLayoutLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var internetIconLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var exitIconLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var entryIconLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var deviceIconLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    val colors =
        HopSelectorDefaults.colors(legendColor = deselectedColor.copy(alpha = expandProgress))
    MotionLayout(
        modifier =
            modifier
                .onGloballyPositioned { motionLayoutLC = it }
                .drawWithContent {
                    drawContent()

                    val motionLayout = motionLayoutLC ?: return@drawWithContent
                    val internet = internetIconLC ?: return@drawWithContent
                    val entry = entryIconLC ?: return@drawWithContent
                    val exit = exitIconLC ?: return@drawWithContent
                    val device = deviceIconLC ?: return@drawWithContent

                    val legendXPosition = motionLayout.localPositionOf(exit).x + exit.size.width / 2

                    drawVerticalLegend(
                        x = legendXPosition,
                        y1 = internet.bottomIn(motionLayout),
                        y2 = exit.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                    drawVerticalLegend(
                        x = legendXPosition,
                        y1 = exit.bottomIn(motionLayout),
                        y2 = entry.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                    drawVerticalLegend(
                        x = legendXPosition,
                        y1 = entry.bottomIn(motionLayout),
                        y2 = device.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                },
        motionScene = scene,
        progress = expandProgress,
    ) {
        LocationHint(
            modifier = Modifier.layoutId(keyInternet),
            text = "Internet",
            imageVector = Icons.Default.Language,
            colors = colors,
            onIconGloballyPositioned = { internetIconLC = it },
        )
        LocationHint(
            modifier = Modifier.layoutId(keyDevice),
            text = "Your device",
            imageVector = Icons.Default.PhoneAndroid,
            colors = colors,
            onIconGloballyPositioned = { deviceIconLC = it },
        )
        Box(
            Modifier.layoutId(keyPanel)
                .clip(RoundedCornerShape(16.dp))
                .background(colors.panelColor)
                .padding(16.dp)
        ) {}
        Hop(
            modifier =
                Modifier.layoutId(keyExit)
                    .padding(
                        start = 4.dp,
                        top = 4.dp,
                        end = 4.dp,
                        bottom = if (exitErrorText == null) 4.dp else 0.dp,
                    ),
            leadingIcon = Icons.Outlined.LocationOn,
            text = exitLocation,
            selected = exitSelected,
            onSelect = onExitClick,
            isError = exitErrorText != null,
            filters = exitFilters,
            onFilterClick = onExitFilterClick,
            colors = colors,
            onIconGloballyPositioned = { exitIconLC = it },
        )

        Text(
            modifier = Modifier.layoutId(keyExitError).padding(end = 4.dp),
            text = exitErrorText ?: "No error",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
        )
        Hop(
            modifier =
                Modifier.layoutId(keyEntry)
                    .padding(
                        start = 4.dp,
                        end = 4.dp,
                        bottom = if (entryErrorText == null) 4.dp else 0.dp,
                    ),
            leadingIcon = Icons.Outlined.Dns,
            text = entryLocation,
            selected = !exitSelected,
            onSelect = onEntryClick,
            isError = entryErrorText != null,
            filters = entryFilters,
            onFilterClick = onEntryFilterClick,
            colors = colors,
            onIconGloballyPositioned = { entryIconLC = it },
        )

        Text(
            modifier = Modifier.layoutId(keyEntryError).padding(end = 4.dp),
            text = entryErrorText ?: "No error",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
        )
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
                Singlehop("Sweden")

                Singlehop("Germany", filters = 2)

                Singlehop("Norway", errorText = "No relays matching your selection", filters = 2)
            }
        }
    }
}

@OptIn(ExperimentalMotionApi::class)
@Composable
fun Singlehop(
    exitLocation: String,
    errorText: String? = null,
    filters: Int = 0,
    onFilterClick: () -> Unit = {},
    expandProgress: Float = 1f,
) {
    val scene = MotionScene {
        val expandSet =
            constraintSet("expanded") {
                val (internet, exit, exitError, device) =
                    createRefsFor(keyInternet, keyExit, keyExitError, keyDevice)
                createVerticalChain(internet, exit, exitError, device)
                constrain(exitError) {
                    visibility = if (errorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(
                        start = parent.start,
                        end = parent.end,
                        startMargin = 28.dp,
                        endMargin = 4.dp,
                    )
                }
            }

        val collapseSet =
            constraintSet("collapsed") {
                val (internet, exit, exitError, device) =
                    createRefsFor(keyInternet, keyExit, keyExitError, keyDevice)
                constrain(internet) { top.linkTo(parent.top) }
                createVerticalChain(exit, exitError)
                constrain(exitError) {
                    visibility = if (errorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(
                        start = parent.start,
                        end = parent.end,
                        startMargin = 28.dp,
                        endMargin = 4.dp,
                    )
                }
                constrain(device) { bottom.linkTo(parent.bottom) }
            }

        defaultTransition(collapseSet, expandSet) {}
    }
    val colors =
        HopSelectorDefaults.colors(legendColor = deselectedColor.copy(alpha = expandProgress))
    var motionLayoutLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var internetIconLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var exitIconLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    var deviceIconLC by remember { mutableStateOf<LayoutCoordinates?>(null) }
    MotionLayout(
        modifier =
            Modifier.onGloballyPositioned { motionLayoutLC = it }
                .drawWithContent {
                    drawContent()

                    val motionLayout = motionLayoutLC ?: return@drawWithContent
                    val internet = internetIconLC ?: return@drawWithContent
                    val exit = exitIconLC ?: return@drawWithContent
                    val device = deviceIconLC ?: return@drawWithContent

                    val x = 20.dp.toPx()
                    drawVerticalLegend(
                        x = x,
                        y1 = internet.bottomIn(motionLayout),
                        y2 = exit.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                    drawVerticalLegend(
                        x = x,
                        y1 = exit.bottomIn(motionLayout),
                        y2 = device.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                },
        motionScene = scene,
        progress = expandProgress,
    ) {
        LocationHint(
            modifier = Modifier.layoutId(keyInternet),
            text = "Internet",
            imageVector = Icons.Default.Language,
            colors = colors,
            onIconGloballyPositioned = { internetIconLC = it },
        )
        LocationHint(
            modifier = Modifier.layoutId(keyDevice),
            text = "Your device",
            imageVector = Icons.Default.PhoneAndroid,
            onIconGloballyPositioned = { deviceIconLC = it },
            colors = colors,
        )
        Hop(
            modifier = Modifier.layoutId(keyExit).padding(horizontal = 4.dp),
            leadingIcon = Icons.Outlined.LocationOn,
            text = exitLocation,
            selected = true,
            onSelect = {},
            isError = errorText != null,
            filters = filters,
            onFilterClick = onFilterClick,
            colors = colors,
            onIconGloballyPositioned = { exitIconLC = it },
        )
        Text(
            modifier = Modifier.layoutId(keyExitError),
            text = errorText ?: "",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
        )
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
                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    isError = false,
                    onFilterClick = {},
                    filters = 0,
                    modifier = Modifier.fillMaxWidth(),
                )

                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    isError = true,
                    onFilterClick = {},
                    filters = 0,
                    modifier = Modifier.fillMaxWidth(),
                )

                Hop(
                    leadingIcon = Icons.Outlined.Dns,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    isError = true,
                    onFilterClick = {},
                    filters = 0,
                    modifier = Modifier.fillMaxWidth(),
                )

                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = false,
                    onSelect = {},
                    isError = false,
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
    isError: Boolean,
    filters: Int,
    onFilterClick: () -> Unit,
    modifier: Modifier = Modifier,
    onIconGloballyPositioned: (LayoutCoordinates) -> Unit = {},
    colors: HopSelectorColors = HopSelectorDefaults.colors(),
) {
    CompositionLocalProvider(
        LocalContentColor provides
            if (selected) MaterialTheme.colorScheme.onPrimary else deselectedColor
    ) {
        Row(
            modifier =
                modifier
                    .semantics {
                        role = Role.Switch
                        this.selected = selected
                    }
                    .clip(RoundedCornerShape(12.dp))
                    .background(colors.containerColor(selected))
                    .clickable(onClick = onSelect)
                    .border(
                        1.dp,
                        animateColorAsState(if (isError) colors.errorColor else Color.Transparent)
                            .value,
                        RoundedCornerShape(12.dp),
                    ),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                modifier =
                    Modifier.onGloballyPositioned(onIconGloballyPositioned)
                        .padding(horizontal = 4.dp, vertical = 2.dp)
                        .size(24.dp),
                imageVector = if (!isError) leadingIcon else Icons.Default.ErrorOutline,
                tint = colors.leadingIconColor(selected, isError),
                contentDescription = null,
            )
            Text(
                modifier = Modifier.weight(1f),
                text = text,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = SemiBold,
            )
            CompositionLocalProvider(LocalContentColor provides colors.selectedContentColor) {
                FilterButton(onClick = onFilterClick, filters = filters)
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
    onIconGloballyPositioned: (LayoutCoordinates) -> Unit,
) {
    CompositionLocalProvider(LocalContentColor provides colors.legendColor) {
        Row(
            modifier.padding(horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(
                modifier =
                    Modifier.padding(1.dp)
                        .onGloballyPositioned(onIconGloballyPositioned)
                        .padding(2.dp)
                        .size(18.dp),
                imageVector = imageVector,
                contentDescription = null,
            )
            Text(
                modifier = Modifier.weight(1f),
                text = text,
                style = MaterialTheme.typography.bodyMedium,
            )
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
fun FilterButton(
    modifier: Modifier = Modifier,
    filters: Int = 0,
    onClick: () -> Unit = {},
    badgeColor: Color = MaterialTheme.colorScheme.surfaceContainer,
) {
    IconButton(modifier = modifier, onClick = onClick) {
        BadgedBox(
            badge = {
                if (filters > 0) {
                    Badge(containerColor = badgeColor) { Text(filters.toString()) }
                }
            }
        ) {
            Icon(imageVector = Icons.Default.FilterList, contentDescription = null)
        }
    }
}

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
        badgeColor: Color = panelColor,
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

private fun LayoutCoordinates.topIn(layoutCoordinates: LayoutCoordinates) =
    layoutCoordinates.localPositionOf(this).y

private fun LayoutCoordinates.bottomIn(layoutCoordinates: LayoutCoordinates) =
    layoutCoordinates.localPositionOf(this).y + size.height

private fun DrawScope.drawVerticalLegend(x: Float, y1: Float, y2: Float, color: Color) {
    if (y1 < y2) {
        drawLine(
            color = color,
            start = Offset(x = x, y = y1),
            strokeWidth = 1.dp.toPx(),
            end = Offset(x = x, y = y2),
            cap = StrokeCap.Round,
        )
    }
}
