package net.mullvad.mullvadvpn.ui.fragments

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.res.colorResource
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.screen.DeviceListScreen
import net.mullvad.mullvadvpn.ui.LoginFragment
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class DeviceListFragment : Fragment() {

    private val deviceListViewModel by viewModel<DeviceListViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        deviceListViewModel.accountToken = arguments?.getString(ACCOUNT_TOKEN_ARGUMENT_KEY)

        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                val topColor = colorResource(R.color.blue)
                ScaffoldWithTopBar(
                    topBarColor = topColor,
                    statusBarColor = topColor,
                    navigationBarColor = colorResource(id = R.color.darkBlue),
                    onSettingsClicked = this@DeviceListFragment::openSettings,
                    content = {
                        DeviceListScreen(
                            viewModel = deviceListViewModel,
                            onBackClick = this@DeviceListFragment::goBack,
                            onContinueWithLogin = this@DeviceListFragment::openLoginView
                        )
                    }
                )
            }
        }
    }

    private fun openLoginView() {
        val loginFragment = LoginFragment().apply {
            if (deviceListViewModel.accountToken != null) {
                arguments = Bundle().apply {
                    putString(
                        ACCOUNT_TOKEN_ARGUMENT_KEY,
                        deviceListViewModel.accountToken
                    )
                }
            }
        }
        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, loginFragment)
            addToBackStack(null)
            commit()
        }
    }

    private fun goBack() {
        parentActivity()?.onBackPressed()
    }

    private fun parentActivity(): MainActivity? {
        return (context as? MainActivity)
    }

    private fun openSettings() = parentActivity()?.openSettings()
}
