package net.mullvad.mullvadvpn

import java.text.DateFormat

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.content.ClipboardManager
import android.content.ClipData
import android.os.Bundle
import android.support.v4.app.Fragment
import android.support.v4.app.FragmentManager
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast

import org.joda.time.DateTime

class AccountFragment : Fragment() {
    private lateinit var parentActivity: MainActivity

    private lateinit var accountExpiryContainer: View
    private lateinit var accountExpiryDisplay: TextView
    private lateinit var accountNumberContainer: View
    private lateinit var accountNumberDisplay: TextView

    private var updateViewJob: Job? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.account, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }

        view.findViewById<View>(R.id.logout).setOnClickListener { logout() }

        accountExpiryContainer = view.findViewById<View>(R.id.account_expiry_container)
        accountNumberContainer = view.findViewById<View>(R.id.account_number_container)

        accountExpiryDisplay = view.findViewById<TextView>(R.id.account_expiry_display)
        accountNumberDisplay = view.findViewById<TextView>(R.id.account_number_display)

        accountNumberContainer.setOnClickListener { copyAccountNumberToClipboard() }

        return view
    }

    override fun onResume() {
        super.onResume()

        parentActivity.accountCache.onAccountDataChange = { accountNumber, accountExpiry ->
            updateViewJob = updateView(accountNumber, accountExpiry)
        }
    }

    override fun onPause() {
        parentActivity.accountCache.onAccountDataChange = null

        super.onPause()
    }

    private fun updateView(accountNumber: String?, accountExpiry: DateTime?) =
            GlobalScope.launch(Dispatchers.Main) {
        if (accountNumber != null) {
            accountNumberDisplay.setText(accountNumber)
            accountNumberContainer.visibility = View.VISIBLE
        } else {
            accountNumberContainer.visibility = View.INVISIBLE
        }

        if (accountExpiry != null) {
            accountExpiryDisplay.setText(formatExpiry(accountExpiry))
            accountExpiryContainer.visibility = View.VISIBLE
        } else {
            accountExpiryContainer.visibility = View.INVISIBLE
        }
    }

    private fun formatExpiry(expiry: DateTime): String {
        val expiryInstant = expiry.toDate()
        val formatter = DateFormat.getDateTimeInstance()

        return formatter.format(expiryInstant)
    }

    private fun logout() {
        clearAccountNumber()
        clearBackStack()
        goToLoginScreen()
    }

    private fun copyAccountNumberToClipboard() {
        val clipboard =
            parentActivity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clipLabel = parentActivity.resources.getString(R.string.mullvad_account_number)
        val clipData = ClipData.newPlainText(clipLabel, accountNumberDisplay.text)

        clipboard.primaryClip = clipData

        Toast.makeText(parentActivity, R.string.copied_mullvad_account_number, Toast.LENGTH_SHORT)
            .show()
    }

    private fun clearAccountNumber() = GlobalScope.launch(Dispatchers.Default) {
        val daemon = parentActivity.daemon.await()

        daemon.setAccount(null)
    }

    private fun clearBackStack() {
        fragmentManager?.apply {
            val firstEntry = getBackStackEntryAt(0)

            popBackStack(firstEntry.id, FragmentManager.POP_BACK_STACK_INCLUSIVE)
        }
    }

    private fun goToLoginScreen() {
        fragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing
            )
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }
}
