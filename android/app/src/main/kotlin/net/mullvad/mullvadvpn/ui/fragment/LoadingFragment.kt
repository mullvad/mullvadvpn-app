package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.LoadingScreen
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.StatusBarPainter

class LoadingFragment : Fragment(), StatusBarPainter, NavigationBarPainter {
    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                LoadingScreen(this@LoadingFragment::openSettings)
            }
        }
    }

    private fun openSettings() {
        (context as? MainActivity)?.openSettings()
    }
}
