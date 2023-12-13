package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.AutoConnectAndLockdownModeScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme

class AutoConnectAndLockdownModeFragment : BaseFragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    AutoConnectAndLockdownModeScreen(
                        onBackClick = { activity?.onBackPressedDispatcher?.onBackPressed() },
                    )
                }
            }
        }
    }
}
