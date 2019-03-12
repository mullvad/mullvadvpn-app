package net.mullvad.mullvadvpn

import android.os.Bundle
import android.support.v4.app.FragmentActivity

class MainActivity: FragmentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main)
    }
}
