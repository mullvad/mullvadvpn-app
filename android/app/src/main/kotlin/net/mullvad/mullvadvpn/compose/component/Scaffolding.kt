package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import kotlinx.coroutines.delay
import me.onebone.toolbar.CollapsingToolbarScaffold
import me.onebone.toolbar.CollapsingToolbarScaffoldScope
import me.onebone.toolbar.CollapsingToolbarScaffoldState
import me.onebone.toolbar.CollapsingToolbarScope
import me.onebone.toolbar.ExperimentalToolbarApi
import me.onebone.toolbar.ScrollStrategy
import net.mullvad.mullvadvpn.R

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScaffoldWithTopBar(
    topBarColor: Color,
    statusBarColor: Color,
    navigationBarColor: Color,
    onSettingsClicked: (() -> Unit)?,
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
    backgroundColor: Color,
    modifier: Modifier = Modifier,
    state: CollapsingToolbarScaffoldState,
    scrollStrategy: ScrollStrategy,
    isEnabledWhenCollapsable: Boolean = true,
    toolbarModifier: Modifier = Modifier,
    toolbar: @Composable CollapsingToolbarScope.() -> Unit,
    body: @Composable CollapsingToolbarScaffoldScope.() -> Unit
) {
    val context = LocalContext.current
    val systemUiController = rememberSystemUiController()
    systemUiController.setNavigationBarColor(backgroundColor)

    var isCollapsable by remember { mutableStateOf(false) }

    LaunchedEffect(isCollapsable) {
        if (!isCollapsable) {
            state.toolbarState.expand()
        }
    }
    LaunchedEffect(isCollapsable) {
        delay(context.resources.getInteger(R.integer.transition_animation_duration).toLong())
        systemUiController.setStatusBarColor(backgroundColor)
    }

    CollapsingToolbarScaffold(
        modifier = modifier.background(backgroundColor),
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
                val minMaxToolbarHeightDiff =
                    with(state) { toolbarState.maxHeight - toolbarState.minHeight }
                val isContentHigherThanCollapseThreshold =
                    with(LocalDensity.current) {
                        bodyHeight > maxHeight.toPx() - minMaxToolbarHeightDiff
                    }
                isCollapsable = isContentHigherThanCollapseThreshold
                body()
            }
        }
    )
}
