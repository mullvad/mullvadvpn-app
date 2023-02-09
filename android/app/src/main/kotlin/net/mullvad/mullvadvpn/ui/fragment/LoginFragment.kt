package net.mullvad.mullvadvpn.ui.fragment

import android.graphics.Rect
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ScrollView
import android.widget.TextView
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.LoginState
import net.mullvad.mullvadvpn.ui.NavigationBarPainter
import net.mullvad.mullvadvpn.ui.extension.requireMainActivity
import net.mullvad.mullvadvpn.ui.paintNavigationBar
import net.mullvad.mullvadvpn.ui.widget.AccountInput
import net.mullvad.mullvadvpn.ui.widget.AccountLogin
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.viewmodel.LoginViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class LoginFragment : BaseFragment(), NavigationBarPainter {

    private val loginViewModel: LoginViewModel by viewModel()

    private lateinit var title: TextView
    private lateinit var subtitle: TextView
    private lateinit var loggingInStatus: View
    private lateinit var loggedInStatus: View
    private lateinit var loginFailStatus: View
    private lateinit var accountLogin: AccountLogin
    private lateinit var scrollArea: ScrollView
    private lateinit var background: View
    private lateinit var headerBar: HeaderBar
    private lateinit var input: AccountInput
    private lateinit var createAccountButton: Button

    @Deprecated("Refactor code to instead rely on Lifecycle.")
    private val jobTracker = JobTracker()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        val view = inflater.inflate(R.layout.login, container, false)

        headerBar = view.findViewById(R.id.header_bar)
        title = view.findViewById(R.id.title)
        subtitle = view.findViewById(R.id.subtitle)
        loggingInStatus = view.findViewById(R.id.logging_in_status)
        loggedInStatus = view.findViewById(R.id.logged_in_status)
        loginFailStatus = view.findViewById(R.id.login_fail_status)

        accountLogin = view.findViewById<AccountLogin>(R.id.account_login).apply {
            onLogin = loginViewModel::login
            onClearHistory = loginViewModel::clearAccountHistory
        }

        createAccountButton = view.findViewById(R.id.create_account)
        createAccountButton.setOnClickAction(
            "createAccount",
            jobTracker,
            loginViewModel::createAccount
        )

        scrollArea = view.findViewById(R.id.scroll_area)

        background = view.findViewById<View>(R.id.contents).apply {
            setOnClickListener { requestFocus() }
        }

        scrollToShow(accountLogin)

        loginViewModel.clearState()
        triggerAutoLoginIfAccountTokenPresent()
        input = accountLogin.findViewById(R.id.input)
        return view
    }

    override fun onStart() {
        super.onStart()
        requireMainActivity().backButtonHandler = {
            if (accountLogin.hasFocus) {
                background.requestFocus()
                true
            } else {
                false
            }
        }
        input.onTextChanged.subscribe(this) {
            createAccountButton.isEnabled = it.isEmpty()
        }
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
    }

    override fun onStop() {
        jobTracker.cancelAllJobs()
        requireMainActivity().backButtonHandler = null
        input.onTextChanged.unsubscribe(this)
        super.onStop()
    }

    private fun triggerAutoLoginIfAccountTokenPresent() {
        arguments?.getString(ACCOUNT_TOKEN_ARGUMENT_KEY)?.also { accountToken ->
            accountLogin.setAccountToken(accountToken)
            loginViewModel.login(accountToken)
        }
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            lanuchUpdateAccountHistory()
            launchUpdateUiOnViewModelStateChanges()
        }
    }

    private fun CoroutineScope.lanuchUpdateAccountHistory() = launch {
        loginViewModel.accountHistory.collect { history ->
            accountLogin.accountHistory = history.accountToken()
        }
    }

    private fun CoroutineScope.launchUpdateUiOnViewModelStateChanges() = launch {
        loginViewModel.uiState
            .onEach {
                // Adds a short delay to prevent loading spinner flickering.
                if (it.isLoading().not()) {
                    delay(MINIMUM_LOADING_SPINNER_TIME_MILLIS)
                }
            }
            .collect { uiState -> updateUi(uiState) }
    }

    private fun updateUi(uiState: LoginViewModel.LoginUiState) {
        when (uiState) {
            is LoginViewModel.LoginUiState.Default -> {
                showDefault()
            }

            is LoginViewModel.LoginUiState.Success -> {
                // MainActivity responsible for transition to connect/out-of-time view.
                showLoggedIn()
            }

            is LoginViewModel.LoginUiState.AccountCreated -> {
                // MainActivity responsible for transition to welcome view.
            }

            is LoginViewModel.LoginUiState.CreatingAccount -> {
                showCreatingAccount()
            }

            is LoginViewModel.LoginUiState.Loading -> {
                showLoading()
            }

            is LoginViewModel.LoginUiState.InvalidAccountError -> {
                loginFailure(resources.getString(R.string.login_fail_description))
            }

            is LoginViewModel.LoginUiState.TooManyDevicesError -> {
                showLoading(overrideSpinnerWithErrorIcon = true)
                openDeviceListFragment(uiState.accountToken)
            }

            is LoginViewModel.LoginUiState.TooManyDevicesMissingListError -> {
                loginFailure(context?.getString(R.string.failed_to_fetch_devices))
            }

            is LoginViewModel.LoginUiState.UnableToCreateAccountError -> {
                loginFailure(resources.getString(R.string.failed_to_create_account))
            }

            is LoginViewModel.LoginUiState.OtherError -> {
                loginFailure(resources.getString(R.string.error_occurred))
            }
        }
    }

    private fun openDeviceListFragment(accountToken: String) {

        val deviceFragment = DeviceListFragment().apply {
            arguments = Bundle().apply { putString(ACCOUNT_TOKEN_ARGUMENT_KEY, accountToken) }
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

    private fun showDefault() {
        accountLogin.state = LoginState.Initial
        headerBar.tunnelState = null
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
    }

    private fun showLoading(overrideSpinnerWithErrorIcon: Boolean = false) {
        accountLogin.state = LoginState.InProgress
        headerBar.setSettingsButtonEnabled(false)

        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.logging_in_description)

        loggingInStatus.visibility = if (overrideSpinnerWithErrorIcon == false) {
            View.VISIBLE
        } else {
            View.GONE
        }

        loginFailStatus.visibility = if (overrideSpinnerWithErrorIcon == false) {
            View.GONE
        } else {
            View.VISIBLE
        }

        loggedInStatus.visibility = View.GONE

        background.requestFocus()

        scrollToShow(loggingInStatus)
    }

    private fun showLoggedIn() {
        title.setText(R.string.logged_in_title)
        subtitle.setText(R.string.logged_in_description)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.VISIBLE

        accountLogin.state = LoginState.Success
        headerBar.setSettingsButtonEnabled(false)

        scrollToShow(loggedInStatus)
    }

    private fun showCreatingAccount() {
        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.creating_new_account)

        loggingInStatus.visibility = View.VISIBLE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.GONE

        accountLogin.state = LoginState.InProgress
        headerBar.setSettingsButtonEnabled(true)

        scrollToShow(loggingInStatus)
    }

    private fun loginFailure(description: String? = "") {
        title.setText(R.string.login_fail_title)
        subtitle.setText(description)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.VISIBLE
        loggedInStatus.visibility = View.GONE

        accountLogin.state = LoginState.Failure
        headerBar.setSettingsButtonEnabled(true)

        scrollToShow(accountLogin)
    }

    private fun scrollToShow(view: View) {
        val rectangle = Rect(0, 0, view.width, view.height)
        scrollArea.requestChildRectangleOnScreen(view, rectangle, false)
    }

    companion object {
        private const val MINIMUM_LOADING_SPINNER_TIME_MILLIS = 200L
    }
}
