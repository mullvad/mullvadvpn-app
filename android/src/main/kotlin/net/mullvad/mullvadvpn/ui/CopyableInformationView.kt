package net.mullvad.mullvadvpn.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.util.AttributeSet
import android.util.TypedValue
import android.view.LayoutInflater
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import net.mullvad.mullvadvpn.R

class CopyableInformationView : LinearLayout {
    enum class WhenMissing {
        Nothing,
        Hide;

        companion object {
            internal fun fromCode(code: Int): WhenMissing {
                when (code) {
                    0 -> return Nothing
                    1 -> return Hide
                    else -> throw Exception("Invalid whenMissing attribute value")
                }
            }
        }
    }

    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.copyable_information_view, this).apply {
                setOnClickListener { copyToClipboard() }
                setEnabled(false)
            }
        }

    private val description: TextView = findViewById(R.id.description)
    private val informationDisplay: TextView = findViewById(R.id.information_display)

    var clipboardLabel: String? = null
        set(value) {
            field = value
            container.setEnabled(clipboardLabel != null)
        }

    var copiedToast: String? = null

    var information: String? = null
        set(value) {
            field = value
            informationDisplay.text = value
            updateStatus()
        }

    var whenMissing = WhenMissing.Nothing
        set(value) {
            field = value
            updateStatus()
        }

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
        val backgroundResource = TypedValue()

        context.theme.resolveAttribute(
            android.R.attr.selectableItemBackground,
            backgroundResource,
            true
        )

        orientation = VERTICAL
        setBackgroundResource(backgroundResource.resourceId)
    }

    private fun loadAttributes(attributes: AttributeSet) {
        val styleableId = R.styleable.CopyableInformationView

        context.theme.obtainStyledAttributes(attributes, styleableId, 0, 0).apply {
            try {
                description.text = getString(R.styleable.CopyableInformationView_description) ?: ""
                clipboardLabel = getString(R.styleable.CopyableInformationView_clipboardLabel)
                copiedToast = getString(R.styleable.CopyableInformationView_copiedToast)
                whenMissing = WhenMissing.fromCode(
                    getInteger(R.styleable.CopyableInformationView_whenMissing, 0)
                )
            } finally {
                recycle()
            }
        }
    }

    private fun updateStatus() {
        when (whenMissing) {
            WhenMissing.Nothing -> visibility = VISIBLE
            WhenMissing.Hide -> {
                if (information == null) {
                    visibility = INVISIBLE
                } else {
                    visibility = VISIBLE
                }
            }
        }
    }

    private fun copyToClipboard() {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clipData = ClipData.newPlainText(clipboardLabel, informationDisplay.text)
        val toastMessage = copiedToast ?: context.getString(R.string.copied_to_clipboard)

        clipboard.primaryClip = clipData

        Toast.makeText(context, toastMessage, Toast.LENGTH_SHORT).show()
    }
}
