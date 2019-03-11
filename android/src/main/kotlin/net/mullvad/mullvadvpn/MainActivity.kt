package net.mullvad.mullvadvpn

import android.os.Bundle
import android.support.v4.app.FragmentActivity

class MainActivity : FragmentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main)

        if (savedInstanceState == null) {
            addInitialFragment()
        }
    }

    private fun addInitialFragment() {
        supportFragmentManager?.beginTransaction()?.apply {
            add(R.id.main_fragment, LoginFragment())
            commit()
        }
    }
}
