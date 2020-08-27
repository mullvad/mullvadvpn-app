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

    private val topLeftCorner: ImageView = container.findViewById(R.id.top_left_corner)
    private val topRightCorner: ImageView = container.findViewById(R.id.top_right_corner)
    private val bottomLeftCorner: ImageView = container.findViewById(R.id.bottom_left_corner)
    private val bottomRightCorner: ImageView = container.findViewById(R.id.bottom_right_corner)

    private val topBorder: ImageView = container.findViewById(R.id.top_border)
    private val leftBorder: ImageView = container.findViewById(R.id.left_border)
    private val rightBorder: ImageView = container.findViewById(R.id.right_border)
    private val bottomBorder: ImageView = container.findViewById(R.id.bottom_border)

    var borderState = BorderState.UNFOCUSED
        set(value) {
            field = value

            when (value) {
                BorderState.UNFOCUSED -> setBorder(unfocusedCorner)
                BorderState.FOCUSED -> setBorder(focusedCorner)
                BorderState.ERROR -> setBorder(errorCorner)
            }
        }

    init {
        val borderElevation = elevation + 0.1f

        topLeftCorner.elevation = borderElevation
        topRightCorner.elevation = borderElevation
        bottomLeftCorner.elevation = borderElevation
        bottomRightCorner.elevation = borderElevation

        topBorder.elevation = borderElevation
        leftBorder.elevation = borderElevation
        rightBorder.elevation = borderElevation
        bottomBorder.elevation = borderElevation
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

    private fun setBorder(corner: Drawable) {
        topLeftCorner.setImageDrawable(corner)
        topRightCorner.setImageDrawable(corner)
        bottomLeftCorner.setImageDrawable(corner)
        bottomRightCorner.setImageDrawable(corner)
    }
}
