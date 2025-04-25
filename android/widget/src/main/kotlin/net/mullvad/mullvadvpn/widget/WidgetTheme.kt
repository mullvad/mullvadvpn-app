package net.mullvad.mullvadvpn.widget

import androidx.compose.runtime.Composable
import androidx.glance.GlanceTheme

@Composable
fun WidgetTheme(content: @Composable () -> Unit) {
    // Set dimensions and type scale based on configurations here
    // val dimensions = defaultDimensions

    GlanceTheme(colors = /*ColorProviders(
                scheme = darkColorScheme,
            )*/ GlanceTheme.colors, content = { content() },)
}
