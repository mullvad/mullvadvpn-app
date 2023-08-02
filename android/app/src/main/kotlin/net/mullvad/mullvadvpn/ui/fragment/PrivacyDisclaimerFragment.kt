package net.mullvad.mullvadvpn.ui.fragment

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.PrivacyDisclaimerScreen
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.lib.common.util.appendHideNavOnReleaseBuild
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.StatusBarPainter
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewModel
import org.koin.android.ext.android.inject

class PrivacyDisclaimerFragment : Fragment(), StatusBarPainter, NavigationBarPainter {

    private val privacyDisclaimerViewModel: PrivacyDisclaimerViewModel by inject()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    PrivacyDisclaimerScreen(
                        onPrivacyPolicyLinkClicked = { openPrivacyPolicy() },
                        onAcceptClicked = { handleAcceptedPrivacyDisclaimer() }
                    )
                }
            }
        }
    }

    private fun handleAcceptedPrivacyDisclaimer() {
        privacyDisclaimerViewModel.setPrivacyDisclosureAccepted()
        (activity as? MainActivity)?.initializeStateHandlerAndServiceConnection(
            apiEndpointConfiguration = activity?.intent?.getApiEndpointConfigurationExtras()
        )
    }

    private fun openPrivacyPolicy() {
        val privacyPolicyUrlIntent =
            Intent(
                Intent.ACTION_VIEW,
                Uri.parse(getString(R.string.privacy_policy_url).appendHideNavOnReleaseBuild())
            )
        context?.startActivity(privacyPolicyUrlIntent)
    }
}
