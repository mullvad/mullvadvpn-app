package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import me.onebone.toolbar.CollapsingToolbarScaffold
import me.onebone.toolbar.CollapsingToolbarScaffoldScope
import me.onebone.toolbar.CollapsingToolbarScaffoldState
import me.onebone.toolbar.CollapsingToolbarScope
import me.onebone.toolbar.ExperimentalToolbarApi
import me.onebone.toolbar.ScrollStrategy

@Composable
fun ScaffoldWithTopBar(
    topBarColor: Color,
    statusBarColor: Color,
    navigationBarColor: Color,
    onSettingsClicked: () -> Unit,
    isIconAndLogoVisible: Boolean = true,
    content: @Composable (PaddingValues) -> Unit,
) {
    val systemUiController = rememberSystemUiController()
    systemUiController.setStatusBarColor(statusBarColor)
    systemUiController.setNavigationBarColor(navigationBarColor)

    Scaffold(
        topBar = {
            TopBar(
                backgroundColor = topBarColor,
                onSettingsClicked = onSettingsClicked,
                isIconAndLogoVisible = isIconAndLogoVisible
            )
        },
        content = content
    )
}

@Composable
@OptIn(ExperimentalToolbarApi::class)
fun CollapsableAwareToolbarScaffold(
    modifier: Modifier,
    state: CollapsingToolbarScaffoldState,
    scrollStrategy: ScrollStrategy,
    isEnabledWhenCollapsable: Boolean = true,
    toolbarModifier: Modifier = Modifier,
    toolbar: @Composable CollapsingToolbarScope.() -> Unit,
    body: @Composable CollapsingToolbarScaffoldScope.() -> Unit
) {
    var isCollapsable by remember { mutableStateOf(false) }

    LaunchedEffect(isCollapsable) {
        if (!isCollapsable) {
            state.toolbarState.expand()
        }
    }

    CollapsingToolbarScaffold(
        modifier = modifier,
        state = state,
        scrollStrategy = scrollStrategy,
        enabled = isEnabledWhenCollapsable && isCollapsable,
        toolbarModifier = toolbarModifier,
        toolbar = toolbar,
        body = {
            var bodyHeight by remember { mutableStateOf(0) }

            BoxWithConstraints(
                modifier = Modifier.onGloballyPositioned { bodyHeight = it.size.height }
            ) {
                val minMaxToolbarHeightDiff = with(state) {
                    toolbarState.maxHeight - toolbarState.minHeight
                }
                val isContentHigherThanCollapseThreshold = with(LocalDensity.current) {
                    bodyHeight > maxHeight.toPx() - minMaxToolbarHeightDiff
                }
                isCollapsable = isContentHigherThanCollapseThreshold
                body()
            }
        }
    )
}
