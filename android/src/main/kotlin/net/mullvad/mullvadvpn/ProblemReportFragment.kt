package net.mullvad.mullvadvpn

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

import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport

class ProblemReportFragment : Fragment() {
    private lateinit var problemReport: MullvadProblemReport

    private lateinit var userEmailInput: EditText
    private lateinit var userMessageInput: EditText
    private lateinit var sendButton: Button

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

        userEmailInput = view.findViewById<EditText>(R.id.user_email)
        userMessageInput = view.findViewById<EditText>(R.id.user_message)
        sendButton = view.findViewById<Button>(R.id.send_button)

        sendButton.setOnClickListener { sendReport() }

        setSendButtonEnabled(false)
        userMessageInput.addTextChangedListener(InputWatcher())

        return view
    }

    private fun sendReport() {
        problemReport.userEmail = userEmailInput.text.toString()
        problemReport.userMessage = userMessageInput.text.toString()
        problemReport.send()
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
