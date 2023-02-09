package net.mullvad.mullvadvpn.ui.fragment

import android.content.Context
import android.graphics.Typeface
import android.os.Bundle
import android.text.Editable
import android.text.Spannable
import android.text.SpannableStringBuilder
import android.text.TextWatcher
import android.text.style.ForegroundColorSpan
import android.text.style.StyleSpan
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.ScrollView
import android.widget.TextView
import android.widget.ViewSwitcher
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.ui.CollapsibleTitleController
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.JobTracker

class ProblemReportFragment : BaseFragment() {
    private val jobTracker = JobTracker()

    private var showingEmail by observable(false) { _, oldValue, newValue ->
        if (oldValue != newValue) {
            if (newValue == true) {
                parentActivity.enterSecureScreen(this)
            } else {
                parentActivity.leaveSecureScreen(this)
            }
        }
    }

    private lateinit var parentActivity: MainActivity
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

    private lateinit var editMessageButton: Button
    private lateinit var tryAgainButton: Button

    private lateinit var scrollArea: ScrollView
    private lateinit var titleController: CollapsibleTitleController

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity

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

        editMessageButton = view.findViewById<Button>(R.id.edit_message_button)
        tryAgainButton = view.findViewById<Button>(R.id.try_again_button)

        view.findViewById<Button>(R.id.view_logs).setOnClickListener {
            showLogs()
        }

        sendButton.setOnClickListener {
            jobTracker.newUiJob("sendReport") {
                sendReport(true)
            }
        }

        editMessageButton.setOnClickListener {
            showForm()
        }

        tryAgainButton.setOnClickListener {
            jobTracker.newUiJob("sendReport") {
                sendReport(false)
            }
        }

        userEmailInput.setText(problemReport.userEmail)
        userMessageInput.setText(problemReport.userMessage)

        setSendButtonEnabled(!userMessageInput.text.isEmpty())
        userMessageInput.addTextChangedListener(InputWatcher())

        scrollArea = view.findViewById(R.id.scroll_area)
        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onDestroyView() {
        problemReport.userEmail = userEmailInput.text.toString()
        problemReport.userMessage = userMessageInput.text.toString()
        problemReport.deleteReportFile()

        titleController.onDestroy()

        super.onDestroyView()
    }

    override fun onDetach() {
        showingEmail = false

        super.onDetach()
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

    private suspend fun sendReport(shouldConfirmNoEmail: Boolean) {
        val userEmail = userEmailInput.text.trim().toString()

        problemReport.userEmail = userEmail
        problemReport.userMessage = userMessageInput.text.toString()

        if (!userEmail.isEmpty() || !shouldConfirmNoEmail || confirmSendWithNoEmail()) {
            showSendingScreen()

            if (problemReport.send().await()) {
                clearForm()
                showSuccessScreen(userEmail)
            } else {
                showErrorScreen()
            }
        }
    }

    private suspend fun confirmSendWithNoEmail(): Boolean {
        val confirmation = CompletableDeferred<Boolean>()

        problemReport.confirmNoEmail = confirmation
        showConfirmNoEmailDialog()

        return confirmation.await()
    }

    private fun clearForm() {
        userEmailInput.setText("")
        userMessageInput.setText("")

        problemReport.userEmail = ""
        problemReport.userMessage = ""
    }

    private fun showForm() {
        bodyContainer.displayedChild = 0
    }

    private fun showConfirmNoEmailDialog() {
        val transaction = parentFragmentManager.beginTransaction()

        transaction.addToBackStack(null)

        ConfirmNoEmailDialogFragment().show(transaction, null)
    }

    private fun showSendingScreen() {
        bodyContainer.displayedChild = 1

        sendingSpinner.visibility = View.VISIBLE
        sentSuccessfullyIcon.visibility = View.GONE
        failedToSendIcon.visibility = View.GONE

        sendStatusLabel.visibility = View.VISIBLE
        sendDetailsLabel.visibility = View.GONE
        responseMessageLabel.visibility = View.GONE

        sendStatusLabel.setText(R.string.sending)

        editMessageButton.visibility = View.GONE
        tryAgainButton.visibility = View.GONE
    }

    private fun showSuccessScreen(userEmail: String) {
        sendingSpinner.visibility = View.GONE

        sentSuccessfullyIcon.visibility = View.VISIBLE
        sendStatusLabel.visibility = View.VISIBLE

        if (!userEmail.isEmpty()) {
            showResponseMessage(userEmail)
        }

        showThanksMessage()
        sendStatusLabel.setText(R.string.sent)

        scrollArea.scrollTo(0, titleController.fullCollapseScrollOffset.toInt())
    }

    private fun showThanksMessage() {
        val thanks = parentActivity.getString(R.string.sent_thanks)
        val weWillLookIntoThis = parentActivity.getString(R.string.we_will_look_into_this)

        val colorStyle = ForegroundColorSpan(parentActivity.getColor(R.color.green))

        sendDetailsLabel.text = SpannableStringBuilder("$thanks $weWillLookIntoThis").apply {
            setSpan(colorStyle, 0, thanks.length, Spannable.SPAN_EXCLUSIVE_EXCLUSIVE)
        }

        sendDetailsLabel.visibility = View.VISIBLE
    }

    private fun showResponseMessage(userEmail: String) {
        val responseMessageTemplate = parentActivity.getString(R.string.sent_contact)
        val responseMessage = parentActivity.getString(R.string.sent_contact, userEmail)

        val emailStart = responseMessageTemplate.indexOf('%')
        val emailEndFromStringEnd = responseMessageTemplate.length - (emailStart + 4)
        val emailEnd = responseMessage.length - emailEndFromStringEnd

        val boldStyle = StyleSpan(Typeface.BOLD)
        val colorStyle = ForegroundColorSpan(parentActivity.getColor(R.color.white))

        responseMessageLabel.text = SpannableStringBuilder(responseMessage).apply {
            setSpan(boldStyle, emailStart, emailEnd, Spannable.SPAN_EXCLUSIVE_EXCLUSIVE)
            setSpan(colorStyle, emailStart, emailEnd, Spannable.SPAN_EXCLUSIVE_EXCLUSIVE)
        }

        responseMessageLabel.visibility = View.VISIBLE

        showingEmail = true
    }

    private fun showErrorScreen() {
        sendingSpinner.visibility = View.GONE

        failedToSendIcon.visibility = View.VISIBLE
        sendStatusLabel.visibility = View.VISIBLE
        sendDetailsLabel.visibility = View.VISIBLE

        sendStatusLabel.setText(R.string.failed_to_send)
        sendDetailsLabel.setText(R.string.failed_to_send_details)

        editMessageButton.visibility = View.VISIBLE
        tryAgainButton.visibility = View.VISIBLE

        scrollArea.scrollTo(0, titleController.fullCollapseScrollOffset.toInt())
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
