package net.mullvad.mullvadvpn.test.e2e.interactor

import android.content.Context
import android.content.Intent
import android.view.View
import android.webkit.WebView
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.TestActivity
import net.mullvad.mullvadvpn.test.common.extension.findObjectByCaseInsensitiveText
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.constant.CONN_CHECK_IS_CONNECTED
import net.mullvad.mullvadvpn.test.e2e.constant.CONN_CHECK_URL
class WebViewInteractor(
    private val context: Context,
    private val device: UiDevice
) {
    fun launchWebView(context: Context, url: String) {
        val intent = Intent(context, TestActivity::class.java).apply {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            putExtra("url", url)
        }
        context.startActivity(intent)
    }

    fun launchAndExtractConnCheckState(): ConnCheckState {
        launchWebView(context, CONN_CHECK_URL)
        val webView = device.findObjectWithTimeout(By.clazz(WebView::class.java))
        val stateText = device.findObjectByCaseInsensitiveText("using Mullvad VPN").apply {
            click()
        }

        // Wait for view to expand after click.
        Thread.sleep(1000)

        val wireGuardIpv4ConnectionRow = webView.findObjects(By.clazz(View::class.java))
            .first { it.text?.endsWith("(WireGuard)") == true }
        val wireGuardIpv4Address = wireGuardIpv4ConnectionRow.text.split(" ")[0].trim()
        return ConnCheckState(stateText.text == CONN_CHECK_IS_CONNECTED, wireGuardIpv4Address)
    }

    data class ConnCheckState(
        val isConnected: Boolean,
        val ipAddress: String
    )
}
