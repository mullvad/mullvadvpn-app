package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.graphics.drawable.Drawable
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.widget.FrameLayout
import android.widget.ImageView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.JobTracker

open class Button : FrameLayout {
    enum class ButtonColor {
        Blue,
        Green,
        Red;

        companion object {
            internal fun fromCode(code: Int): ButtonColor {
                when (code) {
                    0 -> return Blue
                    1 -> return Green
                    2 -> return Red
                    else -> throw Exception("Invalid buttonColor attribute value")
                }
            }
        }
    }

    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.button, this)
        }

    private val button = container.findViewById<android.widget.Button>(R.id.button)
    private val spinner: View = container.findViewById(R.id.spinner)
    private val image: ImageView = container.findViewById(R.id.image)

    private var clickJobName: String? = null
    private var onClickAction: (suspend () -> Unit)? = null

    protected var jobTracker: JobTracker? = null

    var buttonColor: ButtonColor = ButtonColor.Blue
        set(value) {
            field = value

            val backgroundResource = when (value) {
                ButtonColor.Blue -> R.drawable.blue_button_background
                ButtonColor.Green -> R.drawable.green_button_background
                ButtonColor.Red -> R.drawable.red_button_background
            }

            button.setBackgroundResource(backgroundResource)
        }

    var detailImage: Drawable? = null
        set(value) {
            field = value

            image.apply {
                if (value == null) {
                    visibility = GONE
                } else {
                    visibility = VISIBLE
                    setImageDrawable(value)
                }
            }
        }

    var label: CharSequence
        get() = button.text
        set(value) {
            button.text = value
        }

    var showSpinner = false

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

    override fun setEnabled(enabled: Boolean) {
        super.setEnabled(enabled)
        button.setEnabled(enabled)

        if (enabled) {
            alpha = 1.0f
        } else {
            alpha = 0.5f
        }
    }

    init {
        button.setOnClickListener {
            jobTracker?.newUiJob(clickJobName!!) {
                setEnabled(false)

                if (showSpinner) {
                    image.visibility = GONE
                    spinner.visibility = VISIBLE
                }

                onClickAction!!.invoke()

                spinner.visibility = GONE

                if (detailImage != null) {
                    image.visibility = VISIBLE
                }

                setEnabled(true)
            }
        }
    }

    fun setOnClickAction(jobName: String, tracker: JobTracker, action: suspend () -> Unit) {
        clickJobName = jobName
        jobTracker = tracker
        onClickAction = action
    }

    fun setText(textResource: Int) {
        button.setText(textResource)
    }

    private fun loadAttributes(attributes: AttributeSet) {
        var styleableId = R.styleable.Button

        context.theme.obtainStyledAttributes(attributes, styleableId, 0, 0).apply {
            try {
                buttonColor = ButtonColor.fromCode(getInteger(R.styleable.Button_buttonColor, 0))
                detailImage = getDrawable(R.styleable.Button_detailImage)
                showSpinner = getBoolean(R.styleable.Button_showSpinner, false)
            } finally {
                recycle()
            }
        }

        context.theme.obtainStyledAttributes(attributes, R.styleable.TextAttribute, 0, 0).apply {
            try {
                button.text = getString(R.styleable.TextAttribute_text) ?: ""
            } finally {
                recycle()
            }
        }
    }
}
