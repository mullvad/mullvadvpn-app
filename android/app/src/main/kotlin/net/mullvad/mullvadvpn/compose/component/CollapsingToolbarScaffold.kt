package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import me.onebone.toolbar.CollapsingToolbarScaffold
import me.onebone.toolbar.CollapsingToolbarScaffoldScope
import me.onebone.toolbar.CollapsingToolbarScaffoldState
import me.onebone.toolbar.CollapsingToolbarScope
import me.onebone.toolbar.ExperimentalToolbarApi
import me.onebone.toolbar.ScrollStrategy

@OptIn(ExperimentalToolbarApi::class)
@Composable
fun CollapsingToolbarScaffold(
    backgroundColor: Color,
    state: CollapsingToolbarScaffoldState,
    modifier: Modifier = Modifier,
    scrollStrategy: ScrollStrategy = ScrollStrategy.ExitUntilCollapsed,
    isEnabledWhenCollapsable: Boolean = true,
    toolbarModifier: Modifier = Modifier,
    toolbar: @Composable CollapsingToolbarScope.() -> Unit,
    body: @Composable CollapsingToolbarScaffoldScope.() -> Unit,
) {
    val dynamic = remember { mutableStateOf(0.dp) }
    val systemUiController = rememberSystemUiController()
    systemUiController.setNavigationBarColor(backgroundColor)

    var isCollapsable by remember { mutableStateOf(false) }

    LaunchedEffect(isCollapsable) {
        if (!isCollapsable) {
            state.toolbarState.expand()
        }
    }

    val totalHeights = remember { mutableStateOf(0.dp) }
    val localDensity = LocalDensity.current

    CollapsingToolbarScaffold(
        modifier =
            modifier
                .background(backgroundColor)
                .fillMaxWidth()
                .fillMaxHeight()
                .onGloballyPositioned { coordinates ->
                    totalHeights.value = with(localDensity) { coordinates.size.height.toDp() }
                },
        state = state,
        scrollStrategy = scrollStrategy,
        toolbarModifier =
            toolbarModifier.onGloballyPositioned { coordinates ->
                with(localDensity) {
                    dynamic.value = totalHeights.value - coordinates.size.height.toDp()
                }
            },
        enabled = isEnabledWhenCollapsable && isCollapsable,
        toolbar = { toolbar() }
    ) {
        var bodyHeight by remember { mutableIntStateOf(0) }

        BoxWithConstraints(
            modifier =
                Modifier.height(dynamic.value).onGloballyPositioned { bodyHeight = it.size.height }
        ) {
            val minMaxToolbarHeightDiff =
                with(state) { toolbarState.maxHeight - toolbarState.minHeight }
            val isContentHigherThanCollapseThreshold =
                with(localDensity) { bodyHeight >= maxHeight.toPx() - minMaxToolbarHeightDiff }
            isCollapsable = isContentHigherThanCollapseThreshold
            body()
        }
    }
}
