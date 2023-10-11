package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.LoginScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.LoginUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.LoginViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class LoginFragment : BaseFragment() {
    private val vm: LoginViewModel by viewModel()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {

        // TODO: Remove this when we have a better solution for login after clearing max devices
        val accountTokenArgument = arguments?.getString(ACCOUNT_TOKEN_ARGUMENT_KEY)
        if (accountTokenArgument != null) {
            // Login and set initial TextField value
            vm.onAccountNumberChange(accountTokenArgument)
            vm.login(accountTokenArgument)
        }

        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val uiState by vm.uiState.collectAsState()
                    LaunchedEffect(Unit) {
                        vm.uiSideEffect.collect {
                            when (it) {
                                LoginUiSideEffect.NavigateToWelcome,
                                LoginUiSideEffect
                                    .NavigateToConnect -> {} // TODO Fix when we redo navigation
                                is LoginUiSideEffect.TooManyDevices -> {
                                    navigateToDeviceListFragment(it.accountToken)
                                }
                            }
                        }
                    }
                    LoginScreen(
                        uiState,
                        vm::login,
                        vm::createAccount,
                        vm::clearAccountHistory,
                        vm::onAccountNumberChange,
                        ::openSettingsView
                    )
                }
            }
        }
    }

    private fun navigateToDeviceListFragment(accountToken: AccountToken) {
        val deviceFragment =
            DeviceListFragment().apply {
                arguments =
                    Bundle().apply { putString(ACCOUNT_TOKEN_ARGUMENT_KEY, accountToken.value) }
            }

        parentFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, deviceFragment)
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }

    private fun openSettingsView() {
        (context as? MainActivity)?.openSettings()
    }
}
