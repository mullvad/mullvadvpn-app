package net.mullvad.mullvadvpn.ui.widget

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.util.AttributeSet
import android.view.View
import android.widget.ImageButton
import android.widget.Toast
import net.mullvad.mullvadvpn.R

class CopyableInformationView : InformationView {
    var clipboardLabel: String? = null

    var copiedToast: String? = null

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {
        loadAttributes(attributes)
    }

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {
        loadAttributes(attributes)
    }

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
        loadAttributes(attributes)
    }

    init {
        findViewById<ImageButton>(R.id.copy_button).apply {
            visibility = View.VISIBLE
            setOnClickListener { copyToClipboard() }
        }
        shouldEnable = false
    }

    private fun loadAttributes(attributes: AttributeSet) {
        val styleableId = R.styleable.CopyableInformationView

        context.theme.obtainStyledAttributes(attributes, styleableId, 0, 0).apply {
            try {
                clipboardLabel = getString(R.styleable.CopyableInformationView_clipboardLabel)
                copiedToast = getString(R.styleable.CopyableInformationView_copiedToast)
            } finally {
                recycle()
            }
        }
    }

    private fun copyToClipboard() {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clipData = ClipData.newPlainText(clipboardLabel, information)
        val toastMessage = copiedToast ?: context.getString(R.string.copied_to_clipboard)

        clipboard.setPrimaryClip(clipData)

        Toast.makeText(context, toastMessage, Toast.LENGTH_SHORT).show()
    }
}
