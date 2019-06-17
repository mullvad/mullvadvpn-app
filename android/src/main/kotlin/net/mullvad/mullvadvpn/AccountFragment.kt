package net.mullvad.mullvadvpn

import java.text.DateFormat

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView

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

        accountExpiryContainer = view.findViewById<View>(R.id.account_expiry_container)
        accountNumberContainer = view.findViewById<View>(R.id.account_number_container)

        accountExpiryDisplay = view.findViewById<TextView>(R.id.account_expiry_display)
        accountNumberDisplay = view.findViewById<TextView>(R.id.account_number_display)

        updateViewJob = updateView()

        return view
    }

    private fun updateView() = GlobalScope.launch(Dispatchers.Main) {
        val accountCache = parentActivity.accountCache
        val accountNumber = accountCache.accountNumber.await()

        if (accountNumber != null) {
            accountNumberDisplay.setText(accountCache.accountNumber.await())
            accountNumberContainer.visibility = View.VISIBLE

            val accountExpiry = accountCache.accountExpiry.await()

            if (accountExpiry != null) {
                accountExpiryDisplay.setText(formatExpiry(accountExpiry))
                accountExpiryContainer.visibility = View.VISIBLE
            }
        }
    }

    private fun formatExpiry(expiry: DateTime): String {
        val expiryInstant = expiry.toDate()
        val formatter = DateFormat.getDateTimeInstance()

        return formatter.format(expiryInstant)
    }
}
