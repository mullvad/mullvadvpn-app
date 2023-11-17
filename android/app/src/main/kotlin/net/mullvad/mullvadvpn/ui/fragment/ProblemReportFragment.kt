package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.ReportProblemScreen
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.ReportProblemViewModel
import org.koin.androidx.viewmodel.ext.android.viewModel

class ProblemReportFragment : BaseFragment() {
    private val vm by viewModel<ReportProblemViewModel>()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {

        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val uiState = vm.uiState.collectAsState().value
                    ReportProblemScreen(
                        uiState,
                        onSendReport = { email, description -> vm.sendReport(email, description) },
                        onDismissNoEmailDialog = vm::dismissConfirmNoEmail,
                        onClearSendResult = vm::clearSendResult,
                        onNavigateToViewLogs = { showLogs() },
                        updateEmail = vm::updateEmail,
                        updateDescription = vm::updateDescription
                    ) {
                        activity?.onBackPressed()
                    }
                }
            }
        }
    }

    private fun showLogs() {
        parentFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_half_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, ViewLogsFragment())
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }
}
