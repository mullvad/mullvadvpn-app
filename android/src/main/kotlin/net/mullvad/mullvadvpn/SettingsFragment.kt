package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.ImageButton
import android.widget.TextView

class SettingsFragment : Fragment() {
    private lateinit var parentActivity: MainActivity
    private lateinit var remainingTimeLabel: RemainingTimeLabel
    private lateinit var appVersionWarning: View
    private lateinit var appVersionLabel: TextView
    private lateinit var appVersionFooter: View

    private var showCurrentVersionJob: Job? = null

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
        view.findViewById<View>(R.id.app_version).setOnClickListener {
            openLink("https://mullvad.net/download/")
        }
        view.findViewById<View>(R.id.report_a_problem).setOnClickListener {
            openSubFragment(ProblemReportFragment())
        }

        remainingTimeLabel = RemainingTimeLabel(parentActivity, view)
        appVersionWarning = view.findViewById(R.id.app_version_warning)
        appVersionLabel = view.findViewById<TextView>(R.id.app_version_label)
        appVersionFooter = view.findViewById(R.id.app_version_footer)

        showCurrentVersionJob = showCurrentVersion()

        return view
    }

    override fun onResume() {
        super.onResume()
        remainingTimeLabel.onResume()
    }

    override fun onPause() {
        remainingTimeLabel.onPause()
        super.onPause()
    }

    override fun onDestroyView() {
        showCurrentVersionJob?.cancel()
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

    private fun openLink(url: String) {
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))

        startActivity(intent)
    }

    private fun showCurrentVersion() = GlobalScope.launch(Dispatchers.Main) {
        val version = parentActivity.currentVersion.await()
        val versionInfoCache = parentActivity.appVersionInfoCache

        appVersionLabel.setText(version)

        if (versionInfoCache.isLatest && versionInfoCache.isSupported) {
            appVersionWarning.visibility = View.GONE
            appVersionFooter.visibility = View.GONE
        } else {
            appVersionWarning.visibility = View.VISIBLE
            appVersionFooter.visibility = View.VISIBLE
        }
    }
}
