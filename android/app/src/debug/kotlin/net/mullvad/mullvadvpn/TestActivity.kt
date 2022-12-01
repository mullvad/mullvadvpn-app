package net.mullvad.mullvadvpn

import android.annotation.SuppressLint
import android.app.Activity
import android.os.Bundle
import android.webkit.WebView
import android.widget.Toast

class TestActivity : Activity() {
    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_test)
        val testWebView: WebView = findViewById(R.id.webview)
        testWebView.settings.javaScriptEnabled = true
        val url = intent.getStringExtra("url")
        if (url != null) {
            testWebView.loadUrl(url)
        } else {
            Toast.makeText(applicationContext, "No url specified!", Toast.LENGTH_SHORT).show()
        }
    }
}
