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

        remainingTimeLabel = RemainingTimeLabel(parentActivity, view)

        return view
    }

    override fun onDestroyView() {
        remainingTimeLabel.onDestroy()
        super.onDestroyView()
    }
}
