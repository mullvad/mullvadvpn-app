package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup

class LaunchFragment : Fragment() {
    private lateinit var accountTokenCheckJob: Deferred<Boolean>
    private lateinit var advanceToNextScreenJob: Job

    override fun onAttach(context: Context) {
        super.onAttach(context)
        accountTokenCheckJob = checkForAccountToken()
        advanceToNextScreenJob = advanceToNextScreen()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ) = inflater.inflate(R.layout.launch, container, false)

    override fun onDestroy() {
        accountTokenCheckJob.cancel()
        advanceToNextScreenJob.cancel()
        super.onDestroy()
    }

    private fun checkForAccountToken() = GlobalScope.async(Dispatchers.Default) {
        val parentActivity = activity as MainActivity
        val settings = parentActivity.asyncSettings.await()

        settings.accountToken != null
    }

    private fun advanceToNextScreen() = GlobalScope.launch(Dispatchers.Main) {
        val accountTokenIsSet = accountTokenCheckJob.await()

        if (accountTokenIsSet) {
            advanceToConnectScreen()
        } else {
            advanceToLoginScreen()
        }
    }

    private fun advanceToLoginScreen() {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }

    private fun advanceToConnectScreen() {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }
}
