package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.text.method.TransformationMethod
import android.util.AttributeSet
import android.util.TypedValue
import android.view.LayoutInflater
import android.view.View
import android.widget.LinearLayout
import android.widget.TextView
import androidx.appcompat.content.res.AppCompatResources
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

open class InformationView : LinearLayout {
    enum class WhenMissing {
        Nothing,
        Hide,
        ShowSpinner;

        companion object {
            internal fun fromCode(code: Int): WhenMissing {
                when (code) {
                    0 -> return Nothing
                    1 -> return Hide
                    2 -> return ShowSpinner
                    else -> throw Exception("Invalid whenMissing attribute value")
                }
            }
        }
    }

    private val container: View =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.information_view, this).apply {
                setOnClickListener { onClick?.invoke() }
                setEnabled(false)
            }
        }

    private val description: TextView = findViewById(R.id.description)
    private val informationDisplay: TextView = findViewById(R.id.information_display)
    private val spinner: View = findViewById(R.id.spinner)
    private val toggleMaskingButton: View = findViewById(R.id.toggle_masking_button)

    var error by observable<String?>(null) { _, _, _ -> updateStatus() }
    var information by observable<String?>(null) { _, _, _ -> updateStatus() }

    var errorColor by observable(context.getColor(R.color.red)) { _, _, _ -> updateStatus() }
    var informationColor by observable(context.getColor(R.color.white)) { _, _, _ ->
        updateStatus()
    }

    var maxLength by observable(0) { _, _, _ -> updateStatus() }
    var whenMissing by observable(WhenMissing.Nothing) { _, _, _ -> updateStatus() }

    var shouldEnable by observable(false) { _, _, _ -> updateEnabled() }

    var onClick by observable<(() -> Unit)?>(null) { _, _, callback ->
        container.setFocusable(callback != null)
    }

    sealed class Masking {
        object None : Masking()
        data class Hide(val transformationMethod: TransformationMethod) : Masking()
        data class Show(val transformationMethod: TransformationMethod) : Masking()
    }

    var informationState by observable<Masking>(Masking.None) { _, _, newState ->
        when (newState) {
            is Masking.Hide -> {
                informationDisplay.transformationMethod = newState.transformationMethod

                toggleMaskingButton.apply {
                    visibility = VISIBLE
                    contentDescription = context.getString(R.string.show_account_number)
                    background = AppCompatResources.getDrawable(context, R.drawable.icon_show)
                }
            }

            is Masking.Show -> {
                informationDisplay.transformationMethod = newState.transformationMethod

                toggleMaskingButton.apply {
                    visibility = VISIBLE
                    contentDescription = context.getString(R.string.hide_account_number)
                    background = AppCompatResources.getDrawable(context, R.drawable.icon_hide)
                }
            }

            is Masking.None -> {
                informationDisplay.transformationMethod = null
                toggleMaskingButton.visibility = INVISIBLE
            }
        }

        updateStatus()
    }

    var onToggleMaskingClicked by observable<(() -> Unit)?>(null) { _, _, callback ->
        toggleMaskingButton.setOnClickListener { callback?.invoke() }
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
        val styleableId = R.styleable.InformationView

        context.theme.obtainStyledAttributes(attributes, styleableId, 0, 0).apply {
            try {
                description.text = getString(R.styleable.InformationView_description) ?: ""

                errorColor = getInteger(R.styleable.InformationView_errorColor, errorColor)
                maxLength = getInteger(R.styleable.InformationView_maxLength, 0)

                informationColor = getInteger(
                    R.styleable.InformationView_informationColor,
                    informationColor
                )

                whenMissing = WhenMissing.fromCode(
                    getInteger(R.styleable.InformationView_whenMissing, 0)
                )
            } finally {
                recycle()
            }
        }
    }

    private fun updateStatus() {
        val information = this.information
        val hasText = information != null || error != null

        if (error != null) {
            informationDisplay.setTextColor(errorColor)
            informationDisplay.text = error
        } else if (information != null) {
            informationDisplay.setTextColor(informationColor)

            if (maxLength == 0 || information.length <= maxLength) {
                informationDisplay.text = information
            } else {
                informationDisplay.text = information.substring(0, maxLength) + "..."
            }
        }

        if (whenMissing == WhenMissing.Hide && !hasText) {
            visibility = INVISIBLE
        } else {
            visibility = VISIBLE
        }

        if (whenMissing == WhenMissing.ShowSpinner && !hasText) {
            spinner.visibility = VISIBLE
            informationDisplay.visibility = INVISIBLE
        } else {
            spinner.visibility = INVISIBLE
            informationDisplay.visibility = VISIBLE
        }

        updateEnabled()
    }

    private fun updateEnabled() {
        setEnabled(shouldEnable && error == null && information != null)
    }
}
