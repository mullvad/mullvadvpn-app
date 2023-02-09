package net.mullvad.mullvadvpn.ui.fragment

import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.EditText
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.JobTracker

class ViewLogsFragment : BaseFragment() {
    private val jobTracker = JobTracker()

    private lateinit var problemReport: MullvadProblemReport

    private lateinit var logArea: EditText

    override fun onAttach(context: Context) {
        super.onAttach(context)

        val parentActivity = context as MainActivity

        problemReport = parentActivity.problemReport
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.view_logs, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            activity?.onBackPressed()
        }

        logArea = view.findViewById<EditText>(R.id.log_area)

        return view
    }

    override fun onStart() {
        super.onStart()

        jobTracker.newUiJob("showLogs") {
            val logs = jobTracker.runOnBackground {
                problemReport.load()
            }

            logArea.setText(logs)
        }
    }
}
