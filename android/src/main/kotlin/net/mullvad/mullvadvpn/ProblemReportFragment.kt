package net.mullvad.mullvadvpn

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup

import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport

class ProblemReportFragment : Fragment() {
    private lateinit var problemReport: MullvadProblemReport

    override fun onAttach(context: Context) {
        super.onAttach(context)

        val parentActivity = context as MainActivity

        problemReport = parentActivity.problemReport
        problemReport.collect()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.problem_report, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            activity?.onBackPressed()
        }

        view.findViewById<View>(R.id.send_button).alpha = 0.5F

        return view
    }
}
