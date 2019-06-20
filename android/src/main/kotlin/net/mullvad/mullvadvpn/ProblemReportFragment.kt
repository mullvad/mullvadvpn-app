package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.text.Editable
import android.text.TextWatcher
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.TextView
import android.widget.ViewSwitcher

import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport

class ProblemReportFragment : Fragment() {
    private lateinit var problemReport: MullvadProblemReport

    private lateinit var bodyContainer: ViewSwitcher
    private lateinit var userEmailInput: EditText
    private lateinit var userMessageInput: EditText
    private lateinit var sendButton: Button

    private lateinit var sendingSpinner: View
    private lateinit var sentSuccessfullyIcon: View
    private lateinit var failedToSendIcon: View

    private lateinit var sendStatusLabel: TextView
    private lateinit var sendDetailsLabel: TextView
    private lateinit var responseMessageLabel: TextView
    private lateinit var responseEmailLabel: TextView

    private var sendReportJob: Job? = null

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

        bodyContainer = view.findViewById<ViewSwitcher>(R.id.body_container)
        userEmailInput = view.findViewById<EditText>(R.id.user_email)
        userMessageInput = view.findViewById<EditText>(R.id.user_message)
        sendButton = view.findViewById<Button>(R.id.send_button)

        sendingSpinner = view.findViewById<View>(R.id.sending_spinner)
        sentSuccessfullyIcon = view.findViewById<View>(R.id.sent_successfully_icon)
        failedToSendIcon = view.findViewById<View>(R.id.failed_to_send_icon)

        sendStatusLabel = view.findViewById<TextView>(R.id.send_status)
        sendDetailsLabel = view.findViewById<TextView>(R.id.send_details)
        responseMessageLabel = view.findViewById<TextView>(R.id.response_message)
        responseEmailLabel = view.findViewById<TextView>(R.id.response_email)

        sendButton.setOnClickListener {
            sendReportJob?.cancel()
            sendReportJob = sendReport()
        }

        setSendButtonEnabled(false)
        userMessageInput.addTextChangedListener(InputWatcher())

        return view
    }

    override fun onDestroyView() {
        sendReportJob?.cancel()
        super.onDestroyView()
    }

    private fun sendReport() = GlobalScope.launch(Dispatchers.Main) {
        val userEmail = userEmailInput.text.toString()

        problemReport.userEmail = userEmail
        problemReport.userMessage = userMessageInput.text.toString()

        bodyContainer.showNext()

        if (problemReport.send().await()) {
            showSuccessScreen(userEmail)
        } else {
            showErrorScreen()
        }
    }

    private fun showSuccessScreen(userEmail: String) {
        sendingSpinner.visibility = View.GONE

        sentSuccessfullyIcon.visibility = View.VISIBLE
        sendStatusLabel.visibility = View.VISIBLE
        sendDetailsLabel.visibility = View.VISIBLE

        if (!userEmail.isEmpty()) {
            responseMessageLabel.visibility = View.VISIBLE
            responseEmailLabel.visibility = View.VISIBLE
            responseEmailLabel.text = userEmail
        }

        sendStatusLabel.setText(R.string.sent)
    }

    private fun showErrorScreen() {
        sendingSpinner.visibility = View.GONE

        failedToSendIcon.visibility = View.VISIBLE
        sendStatusLabel.visibility = View.VISIBLE
        sendDetailsLabel.visibility = View.VISIBLE

        sendStatusLabel.setText(R.string.failed_to_send)
        sendDetailsLabel.setText(R.string.failed_to_send_details)
    }

    private fun setSendButtonEnabled(enabled: Boolean) {
        sendButton.setEnabled(enabled)
        sendButton.alpha = if (enabled) 1.0F else 0.5F
    }

    inner class InputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            setSendButtonEnabled(!text.isEmpty())
        }
    }
}
