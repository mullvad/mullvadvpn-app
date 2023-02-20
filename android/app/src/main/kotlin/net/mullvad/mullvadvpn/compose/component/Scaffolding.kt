package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.google.accompanist.systemuicontroller.rememberSystemUiController

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
