package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.graphics.drawable.Drawable
import android.util.AttributeSet
import android.view.LayoutInflater
import android.widget.ImageView
import android.widget.RelativeLayout
import net.mullvad.mullvadvpn.R

class AccountInputContainer : RelativeLayout {
    enum class BorderState {
        UNFOCUSED,
        FOCUSED,
        ERROR
    }

    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.account_input_container, this)
        }

    private val errorCorner = resources.getDrawable(R.drawable.account_input_corner_error, null)
    private val focusedCorner = resources.getDrawable(R.drawable.account_input_corner_focused, null)
    private val unfocusedCorner = resources.getDrawable(R.drawable.account_input_corner, null)

    private val errorBorder = resources.getDrawable(R.drawable.account_input_border_error, null)
    private val focusedBorder = resources.getDrawable(R.drawable.account_input_border_focused, null)

    private val topLeftCorner: ImageView = container.findViewById(R.id.top_left_corner)
    private val topRightCorner: ImageView = container.findViewById(R.id.top_right_corner)
    private val bottomLeftCorner: ImageView = container.findViewById(R.id.bottom_left_corner)
    private val bottomRightCorner: ImageView = container.findViewById(R.id.bottom_right_corner)

    var borderState = BorderState.UNFOCUSED
        set(value) {
            field = value

            overlay.clear()

            when (value) {
                BorderState.UNFOCUSED -> setBorder(unfocusedCorner)
                BorderState.FOCUSED -> {
                    setBorder(focusedCorner)
                    overlay.add(focusedBorder)
                }
                BorderState.ERROR -> {
                    setBorder(errorCorner)
                    overlay.add(errorBorder)
                }
            }
        }

    init {
        val borderElevation = elevation + 0.1f

        topLeftCorner.elevation = borderElevation
        topRightCorner.elevation = borderElevation
        bottomLeftCorner.elevation = borderElevation
        bottomRightCorner.elevation = borderElevation
    }

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
    }

    protected override fun onLayout(
        changed: Boolean,
        left: Int,
        top: Int,
        right: Int,
        bottom: Int
    ) {
        super.onLayout(changed, left, top, right, bottom)

        if (changed) {
            val width = right - left
            val height = bottom - top

            errorBorder.setBounds(0, 0, width, height)
            focusedBorder.setBounds(0, 0, width, height)
        }
    }

    private fun setBorder(corner: Drawable) {
        topLeftCorner.setImageDrawable(corner)
        topRightCorner.setImageDrawable(corner)
        bottomLeftCorner.setImageDrawable(corner)
        bottomRightCorner.setImageDrawable(corner)
    }
}
