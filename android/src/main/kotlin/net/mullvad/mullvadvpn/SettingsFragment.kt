package net.mullvad.mullvadvpn

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.ImageButton

class SettingsFragment : Fragment() {
    private lateinit var parentActivity: MainActivity
    private lateinit var remainingTimeLabel: RemainingTimeLabel

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.settings, container, false)

        view.findViewById<ImageButton>(R.id.close).setOnClickListener {
            activity?.onBackPressed()
        }

        view.findViewById<Button>(R.id.quit_button).setOnClickListener {
            activity?.finishAndRemoveTask()
        }

        view.findViewById<View>(R.id.account).setOnClickListener {
            openSubFragment(AccountFragment())
        }
        view.findViewById<View>(R.id.report_a_problem).setOnClickListener {
            openSubFragment(ProblemReportFragment())
        }

        remainingTimeLabel = RemainingTimeLabel(parentActivity, view)

        return view
    }

    override fun onResume() {
        super.onResume()
        remainingTimeLabel.onResume()
    }

    override fun onDestroyView() {
        remainingTimeLabel.onDestroy()
        super.onDestroyView()
    }

    private fun openSubFragment(fragment: Fragment) {
        fragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_half_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, fragment)
            addToBackStack(null)
            commit()
        }
    }
}
