package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Toast
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.res.colorResource
import androidx.fragment.app.Fragment
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.flowWithLifecycle
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.screen.DeviceListScreen
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class DeviceListFragment : Fragment() {

    private val deviceListViewModel by viewModel<DeviceListViewModel>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

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
                            onBackClick = { openLoginView(doTriggerAutoLogin = false) },
                            onContinueWithLogin = { openLoginView(doTriggerAutoLogin = true) }
                        )
                    }
                )
            }
        }
    }

    override fun onResume() {
        super.onResume()
        deviceListViewModel.clearStagedDevice()
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        deviceListViewModel.toastMessages
            .flowWithLifecycle(lifecycle, Lifecycle.State.RESUMED)
            .collect {
                Toast.makeText(context, it, Toast.LENGTH_SHORT).show()
            }
    }

    private fun openLoginView(doTriggerAutoLogin: Boolean) {
        parentActivity()?.clearBackStack()
        val loginFragment = LoginFragment().apply {
            if (doTriggerAutoLogin && deviceListViewModel.accountToken != null) {
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
            commitAllowingStateLoss()
        }
    }

    private fun parentActivity(): MainActivity? {
        return (context as? MainActivity)
    }

    private fun openSettings() = parentActivity()?.openSettings()
}
