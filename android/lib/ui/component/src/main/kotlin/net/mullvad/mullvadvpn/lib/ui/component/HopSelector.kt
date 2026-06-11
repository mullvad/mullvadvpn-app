package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.EaseInQuint
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AddLocationAlt
import androidx.compose.material.icons.outlined.LocationOn
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.FilterList
import androidx.compose.material.icons.rounded.FilterListOff
import androidx.compose.material.icons.rounded.Language
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.minimumInteractiveComponentSize
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
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
import net.mullvad.mullvadvpn.lib.model.FilterTarget
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.tag.HOP_SELECTOR_ENTRY_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaDisabled

private object AnimationKey {
    const val Internet = "internet"
    const val Panel = "panel"
    const val Exit = "exit"
    const val ExitError = "exitError"
    const val Entry = "entry"
    const val EntryError = "entryError"
    const val Device = "device"
}

@OptIn(ExperimentalSharedTransitionApi::class)
@Preview
@Composable
private fun PreviewCollapsibleSinglehopSelector() {

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
private fun PreviewCollapsibleMultihopSelector() {
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
                userLocation = "Denmark",
                exitSelected = exitSelected,
                entryLocation = "Germany",
                entryErrorText = if (isEntryError) "No relays matching your selection" else null,
                onEntryClick = { exitSelected = false },
                exitLocation = "Sweden",
                exitErrorText = if (isExitError) "No relays matching your selection" else null,
                onExitClick = { exitSelected = true },
                expandProgress = progress,
            )
        }
    }
}

@OptIn(ExperimentalSharedTransitionApi::class)
@PreviewFontScale
@Preview
@Composable
private fun PreviewMultihopSelector() {
    AppTheme {
        Surface {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                MultihopSelector(
                    userLocation = "USA",
                    exitSelected = false,
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                )
                MultihopSelector(
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                    entryErrorText = null,
                    exitErrorText = "No relays matching your selection",
                )

                MultihopSelector(
                    entryLocation = "Sweden",
                    exitLocation = "Germany",
                    exitErrorText = null,
                    entryErrorText = "No relays matching your selection",
                )
            }
        }
    }
}

enum class FilterButtonState {
    Enabled,
    EnabledFiltersActive,
    Disabled,
}

@OptIn(ExperimentalSharedTransitionApi::class, ExperimentalMotionApi::class)
@Suppress("LongMethod")
@Composable
fun MultihopSelector(
    modifier: Modifier = Modifier,
    userLocation: String? = null,
    exitSelected: Boolean = true,
    entryLocation: String,
    entryErrorText: String? = null,
    entryFilterButtonState: FilterButtonState = FilterButtonState.Enabled,
    onEntryClick: () -> Unit = {},
    exitLocation: String,
    exitErrorText: String? = null,
    exitFilterButtonState: FilterButtonState = FilterButtonState.Enabled,
    onExitClick: () -> Unit = {},
    onFilterClick: (FilterTarget) -> Unit = {},
    expandProgress: Float = 1f,
) {
    val scene = MotionScene {
        val expandSet =
            constraintSet("expanded") {
                val (internet, exit, exitError, entry, entryError, device, panel) =
                    createRefsFor(
                        AnimationKey.Internet,
                        AnimationKey.Exit,
                        AnimationKey.ExitError,
                        AnimationKey.Entry,
                        AnimationKey.EntryError,
                        AnimationKey.Device,
                        AnimationKey.Panel,
                    )
                createVerticalChain(device, entry, entryError, exit, exitError, internet)
                constrain(exit) { linkTo(start = parent.start, end = parent.end) }
                constrain(entry) { linkTo(start = parent.start, end = parent.end) }
                constrain(exitError) {
                    visibility = if (exitErrorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(start = parent.start, end = parent.end)
                }
                constrain(entryError) {
                    visibility = if (entryErrorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(start = parent.start, end = parent.end)
                }
                constrain(panel) {
                    width = Dimension.fillToConstraints
                    height = Dimension.fillToConstraints
                    linkTo(
                        top = entry.top,
                        bottom = exitError.bottom,
                        start = entry.start,
                        end = entry.end,
                    )
                }
            }

        val collapseSet =
            constraintSet("collapsed") {
                val (internet, exit, exitError, entry, entryError, device, panel) =
                    createRefsFor(
                        AnimationKey.Internet,
                        AnimationKey.Exit,
                        AnimationKey.ExitError,
                        AnimationKey.Entry,
                        AnimationKey.EntryError,
                        AnimationKey.Device,
                        AnimationKey.Panel,
                    )

                constrain(device) { top.linkTo(parent.top) }
                constrain(internet) { bottom.linkTo(parent.bottom) }

                createVerticalChain(entry, entryError, exit, exitError)

                constrain(exit) { linkTo(start = parent.start, end = parent.end) }
                constrain(entry) { linkTo(start = parent.start, end = parent.end) }

                constrain(exitError) {
                    visibility = if (exitErrorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(start = parent.start, end = parent.end)
                }
                constrain(entryError) {
                    visibility = if (entryErrorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(start = parent.start, end = parent.end)
                }

                constrain(panel) {
                    width = Dimension.fillToConstraints
                    height = Dimension.fillToConstraints
                    linkTo(
                        top = entry.top,
                        bottom = exitError.bottom,
                        start = entry.start,
                        end = entry.end,
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
        HopSelectorDefaults.colors(
            legendColor = deselectedColor.copy(alpha = EaseInQuint.transform(expandProgress)),
            hintColor = deselectedColor.copy(alpha = expandProgress),
        )
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
                        y1 = device.bottomIn(motionLayout),
                        y2 = entry.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                    drawVerticalLegend(
                        x = legendXPosition,
                        y1 = entry.bottomIn(motionLayout),
                        y2 = exit.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                    drawVerticalLegend(
                        x = legendXPosition,
                        y1 = exit.bottomIn(motionLayout),
                        y2 = internet.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                },
        motionScene = scene,
        progress = expandProgress,
    ) {
        LocationHint(
            modifier = Modifier.layoutId(AnimationKey.Device),
            text = resolveYourDeviceText(userLocation),
            imageVector = Icons.Rounded.PhoneAndroid,
            colors = colors,
            onIconGloballyPositioned = { deviceIconLC = it },
        )
        LocationHint(
            modifier = Modifier.layoutId(AnimationKey.Internet),
            text = stringResource(R.string.internet),
            imageVector = Icons.Rounded.Language,
            colors = colors,
            onIconGloballyPositioned = { internetIconLC = it },
        )
        Box(
            Modifier.layoutId(AnimationKey.Panel)
                .clip(RoundedCornerShape(Dimens.multihopSelectorPanelRadius))
                .background(colors.panelColor)
        ) {}
        Hop(
            modifier =
                Modifier.testTag(HOP_SELECTOR_ENTRY_TEST_TAG)
                    .layoutId(AnimationKey.Entry)
                    .padding(
                        top = 4.dp,
                        start = 4.dp,
                        end = 4.dp,
                        bottom = if (entryErrorText == null) Dimens.tinyPadding else 0.dp,
                    ),
            leadingIcon =
                if (entryFilterButtonState == FilterButtonState.Disabled) MultihopWhenNeeded
                else Icons.Outlined.AddLocationAlt,
            text = entryLocation,
            selected = !exitSelected,
            onSelect = onEntryClick,
            isError = entryErrorText != null,
            colors = colors,
            filterButtonState = entryFilterButtonState,
            onFilterClick = { onFilterClick(FilterTarget.Entry) },
            onIconGloballyPositioned = { entryIconLC = it },
        )

        Text(
            modifier =
                Modifier.layoutId(AnimationKey.EntryError)
                    .padding(
                        start = Dimens.hopSelectorErrorStartPadding,
                        end = Dimens.smallPadding,
                    ),
            text = entryErrorText ?: "No error",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
        )
        Hop(
            modifier =
                Modifier.layoutId(AnimationKey.Exit)
                    .padding(
                        start = 4.dp,
                        end = 4.dp,
                        bottom = if (exitErrorText == null) 4.dp else 0.dp,
                    ),
            leadingIcon = Icons.Outlined.LocationOn,
            text = exitLocation,
            selected = exitSelected,
            onSelect = onExitClick,
            isError = exitErrorText != null,
            colors = colors,
            filterButtonState = exitFilterButtonState,
            onFilterClick = { onFilterClick(FilterTarget.Exit) },
            onIconGloballyPositioned = { exitIconLC = it },
        )
        Text(
            modifier =
                Modifier.layoutId(AnimationKey.ExitError)
                    .padding(
                        start = Dimens.hopSelectorErrorStartPadding,
                        end = Dimens.smallPadding,
                        bottom = Dimens.tinyPadding,
                    ),
            text = exitErrorText ?: "No error",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
        )
    }
}

@Preview
@Composable
private fun PreviewSinglehopSelector() {
    AppTheme {
        Surface {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                Singlehop("Sweden")

                Singlehop("Germany")

                Singlehop("Norway", errorText = "No relays matching your selection")
            }
        }
    }
}

@OptIn(ExperimentalMotionApi::class)
@Suppress("LongMethod")
@Composable
fun Singlehop(
    exitLocation: String,
    userLocation: String? = null,
    errorText: String? = null,
    expandProgress: Float = 1f,
    filterButtonState: FilterButtonState = FilterButtonState.Enabled,
    onFilterClick: () -> Unit = {},
    onSelect: (() -> Unit) = {},
) {
    val scene = MotionScene {
        val expandSet =
            constraintSet("expanded") {
                val (internet, exit, exitError, device) =
                    createRefsFor(
                        AnimationKey.Internet,
                        AnimationKey.Exit,
                        AnimationKey.ExitError,
                        AnimationKey.Device,
                    )
                createVerticalChain(device, exit, exitError, internet)
                constrain(exitError) {
                    visibility = if (errorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(start = parent.start, end = parent.end)
                }
            }

        val collapseSet =
            constraintSet("collapsed") {
                val (internet, exit, exitError, device) =
                    createRefsFor(
                        AnimationKey.Internet,
                        AnimationKey.Exit,
                        AnimationKey.ExitError,
                        AnimationKey.Device,
                    )
                constrain(device) { top.linkTo(parent.top) }
                createVerticalChain(exit, exitError)
                constrain(exitError) {
                    visibility = if (errorText == null) Visibility.Gone else Visibility.Visible
                    width = Dimension.fillToConstraints
                    linkTo(start = parent.start, end = parent.end)
                }
                constrain(internet) { bottom.linkTo(parent.bottom) }
            }

        defaultTransition(collapseSet, expandSet) {}
    }
    val colors =
        HopSelectorDefaults.colors(
            legendColor = deselectedColor.copy(alpha = EaseInQuint.transform(expandProgress)),
            hintColor = deselectedColor.copy(alpha = expandProgress),
        )
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

                    val legendXPosition = motionLayout.localPositionOf(exit).x + exit.size.width / 2
                    drawVerticalLegend(
                        x = legendXPosition,
                        y1 = device.bottomIn(motionLayout),
                        y2 = exit.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                    drawVerticalLegend(
                        x = legendXPosition,
                        y1 = exit.bottomIn(motionLayout),
                        y2 = internet.topIn(motionLayout),
                        color = colors.legendColor,
                    )
                },
        motionScene = scene,
        progress = expandProgress,
    ) {
        LocationHint(
            modifier = Modifier.layoutId(AnimationKey.Device),
            text = resolveYourDeviceText(userLocation),
            imageVector = Icons.Rounded.PhoneAndroid,
            onIconGloballyPositioned = { deviceIconLC = it },
            colors = colors,
        )
        LocationHint(
            modifier = Modifier.layoutId(AnimationKey.Internet),
            text = stringResource(R.string.internet),
            imageVector = Icons.Rounded.Language,
            colors = colors,
            onIconGloballyPositioned = { internetIconLC = it },
        )
        Hop(
            modifier =
                Modifier.layoutId(AnimationKey.Exit).padding(horizontal = Dimens.tinyPadding),
            leadingIcon = Icons.Outlined.LocationOn,
            text = exitLocation,
            selected = true,
            onSelect = onSelect,
            isError = errorText != null,
            colors = colors,
            onIconGloballyPositioned = { exitIconLC = it },
            filterButtonState = filterButtonState,
            onFilterClick = onFilterClick,
        )
        Text(
            modifier =
                Modifier.layoutId(AnimationKey.ExitError)
                    .padding(Dimens.hopSelectorErrorStartPadding, end = Dimens.tinyPadding),
            text = errorText ?: "",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
        )
    }
}

@Preview
@Composable
private fun PreviewHop() {
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
                    modifier = Modifier.fillMaxWidth(),
                    filterButtonState = FilterButtonState.Enabled,
                    onFilterClick = {},
                )

                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    isError = true,
                    modifier = Modifier.fillMaxWidth(),
                    filterButtonState = FilterButtonState.Enabled,
                    onFilterClick = {},
                )

                Hop(
                    leadingIcon = Icons.Outlined.AddLocationAlt,
                    text = "Sweden",
                    selected = true,
                    onSelect = {},
                    isError = true,
                    modifier = Modifier.fillMaxWidth(),
                    filterButtonState = FilterButtonState.Enabled,
                    onFilterClick = {},
                )

                Hop(
                    leadingIcon = Icons.Outlined.LocationOn,
                    text = "Sweden",
                    selected = false,
                    onSelect = {},
                    isError = false,
                    modifier = Modifier.fillMaxWidth(),
                    filterButtonState = FilterButtonState.Disabled,
                    onFilterClick = {},
                )
            }
        }
    }
}

@OptIn(ExperimentalMotionApi::class)
@Composable
private fun Hop(
    leadingIcon: ImageVector,
    filterButtonState: FilterButtonState,
    onFilterClick: () -> Unit,
    text: String,
    selected: Boolean,
    onSelect: (() -> Unit)?,
    isError: Boolean,
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
                    .clip(RoundedCornerShape(Dimens.hopRadius))
                    .background(colors.containerColor(selected))
                    .let {
                        if (onSelect != null) {
                            it.clickable(onClick = onSelect)
                        } else {
                            it
                        }
                    }
                    .minimumInteractiveComponentSize()
                    .border(
                        1.dp,
                        animateColorAsState(if (isError) colors.errorColor else Color.Transparent)
                            .value,
                        RoundedCornerShape(Dimens.hopRadius),
                    ),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                modifier =
                    Modifier.onGloballyPositioned(onIconGloballyPositioned)
                        .padding(
                            horizontal = Dimens.tinyPadding,
                            vertical = Dimens.hopIconVerticalInternalPadding,
                        )
                        .size(Dimens.hopIconSize),
                imageVector = if (!isError) leadingIcon else Icons.Rounded.ErrorOutline,
                tint = colors.leadingIconColor(selected, isError),
                contentDescription = null,
            )
            Text(
                modifier = Modifier.weight(1f),
                text = text,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = SemiBold,
            )
            FilterButton(onFilterClick = onFilterClick, filterButtonState = filterButtonState)
        }
    }
}

@Composable
private fun FilterButton(onFilterClick: () -> Unit, filterButtonState: FilterButtonState) {
    IconButton(
        modifier = Modifier.padding(end = Dimens.smallPadding),
        onClick = onFilterClick,
        enabled = filterButtonState != FilterButtonState.Disabled,
    ) {
        Icon(
            imageVector =
                when (filterButtonState) {
                    FilterButtonState.Enabled -> Icons.Rounded.FilterList
                    // add correct filters enabled icon here
                    FilterButtonState.EnabledFiltersActive -> Icons.Rounded.FilterList
                    FilterButtonState.Disabled -> Icons.Rounded.FilterListOff
                },
            tint =
                if (filterButtonState == FilterButtonState.Disabled)
                    MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled)
                else LocalContentColor.current,
            contentDescription = stringResource(R.string.filter),
        )
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
    CompositionLocalProvider(LocalContentColor provides colors.hintColor) {
        Row(
            modifier.padding(horizontal = Dimens.smallPadding),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Dimens.smallPadding),
        ) {
            Icon(
                modifier =
                    Modifier.padding(1.dp)
                        .onGloballyPositioned(onIconGloballyPositioned)
                        .padding(Dimens.locationHintInternalPadding)
                        .size(Dimens.locationHintIconSize),
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

@Composable
fun resolveYourDeviceText(userLocation: String?): String =
    if (userLocation != null) {
        stringResource(R.string.your_device_with_location, userLocation)
    } else {
        stringResource(R.string.your_device)
    }

@Immutable
class HopSelectorColors(
    val selectedContentColor: Color,
    val deselectedContentColor: Color,
    val selectedContainerColor: Color,
    val deselectedContainerColor: Color,
    val panelColor: Color,
    val errorColor: Color,
    val hintColor: Color,
    val legendColor: Color,
) {
    @Stable
    internal fun containerColor(selected: Boolean): Color =
        if (selected) selectedContainerColor else deselectedContainerColor

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
        selectedContainerColor: Color = MaterialTheme.colorScheme.primaryContainer,
        deselectedContainerColor: Color = Color.Transparent,
        panelColor: Color = MaterialTheme.colorScheme.tertiaryContainer,
        errorColor: Color = MaterialTheme.colorScheme.error,
        hintColor: Color = deselectedColor,
        legendColor: Color = deselectedColor,
    ): HopSelectorColors =
        HopSelectorColors(
            selectedContentColor,
            deselectedContentColor,
            selectedContainerColor,
            deselectedContainerColor,
            panelColor,
            errorColor,
            hintColor,
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
