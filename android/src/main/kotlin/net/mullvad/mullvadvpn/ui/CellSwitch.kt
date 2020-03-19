package net.mullvad.mullvadvpn.ui

import android.animation.ValueAnimator
import android.content.Context
import android.graphics.Paint.Style
import android.graphics.drawable.ShapeDrawable
import android.graphics.drawable.shapes.OvalShape
import android.util.AttributeSet
import android.view.GestureDetector
import android.view.GestureDetector.OnGestureListener
import android.view.Gravity
import android.view.MotionEvent
import android.widget.ImageView
import android.widget.LinearLayout
import net.mullvad.mullvadvpn.R

class CellSwitch : LinearLayout {
    enum class State {
        ON,
        OFF;

        val isOn
            get() = this == State.ON
        val isOff
            get() = this == State.OFF

        companion object {
            fun fromBoolean(on: Boolean): State {
                if (on) {
                    return State.ON
                } else {
                    return State.OFF
                }
            }
        }
    }

    var state = State.OFF
        set(value) {
            if (field != value) {
                field = value
                animateToState()
                listener?.invoke(value)
            }
        }

    var listener: ((State) -> Unit)? = null

    private val onColor = resources.getColor(R.color.green)
    private val offColor = resources.getColor(R.color.red)

    private val knobSize = resources.getDimensionPixelSize(R.dimen.cell_switch_knob_size)
    private val knobImage = ShapeDrawable(OvalShape()).apply {
        paint.apply {
            color = offColor
            style = Style.FILL
        }

        intrinsicWidth = knobSize
        intrinsicHeight = knobSize
    }

    private val knobView = ImageView(context).apply {
        setImageDrawable(knobImage)
    }

    private val knobAnimationDuration = 200L
    private val knobMaxTranslation =
        resources.getDimensionPixelOffset(R.dimen.cell_switch_knob_max_translation).toFloat()

    private val knobPosition: Float
        get() = knobView.translationX / knobMaxTranslation

    private val positionAnimation = ValueAnimator.ofFloat(0f, knobMaxTranslation).apply {
        addUpdateListener { animation ->
            knobView.translationX = animation.animatedValue as Float
        }

        duration = knobAnimationDuration
    }

    private val colorAnimation = ValueAnimator.ofArgb(offColor, onColor).apply {
        addUpdateListener { animation ->
            knobImage.paint.color = animation.animatedValue as Int
            knobImage.invalidateSelf()
        }

        duration = knobAnimationDuration
    }

    private val gestureListener = object : OnGestureListener {
        private var isScrolling: Boolean = false
        private var scrollPosition: Float = 0f

        override fun onDown(event: MotionEvent): Boolean {
            scrollPosition = knobView.translationX
            return true
        }

        override fun onFling(
            downEvent: MotionEvent,
            upEvent: MotionEvent,
            velocityX: Float,
            velocityY: Float
        ): Boolean {
            if (velocityX > 0f) {
                state = State.ON
            } else if (velocityX < 0f) {
                state = State.OFF
            }

            return true
        }

        override fun onLongPress(event: MotionEvent) {}

        override fun onScroll(
            downEvent: MotionEvent,
            moveEvent: MotionEvent,
            distanceX: Float,
            distanceY: Float
        ): Boolean {
            isScrolling = true
            scrollPosition -= distanceX

            var fraction = scrollPosition / knobMaxTranslation
            val playTime = (fraction * knobAnimationDuration).toLong()

            colorAnimation.pause()
            positionAnimation.pause()

            colorAnimation.currentPlayTime = playTime
            positionAnimation.currentPlayTime = playTime

            return true
        }

        override fun onShowPress(event: MotionEvent) {}

        override fun onSingleTapUp(event: MotionEvent): Boolean {
            when (state) {
                State.ON -> state = State.OFF
                State.OFF -> state = State.ON
            }

            return true
        }

        fun onUp(): Boolean {
            if (!isScrolling) {
                return false
            }

            if (knobPosition <= 0.5f) {
                state = State.OFF
            } else {
                state = State.ON
            }

            isScrolling = false
            scrollPosition = 0f

            return true
        }
    }

    private val gestureDetector = GestureDetector(context, gestureListener)

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {}

    init {
        setBackground(resources.getDrawable(R.drawable.cell_switch_background, null))
        addView(knobView, LinearLayout.LayoutParams(knobSize, knobSize).apply {
            gravity = Gravity.CENTER_VERTICAL
            leftMargin = resources.getDimensionPixelSize(R.dimen.cell_switch_knob_margin)
        })
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (gestureDetector.onTouchEvent(event)) {
            return true
        } else if (event.actionMasked == MotionEvent.ACTION_UP) {
            return gestureListener.onUp()
        }

        return super.onTouchEvent(event)
    }

    fun forcefullySetState(newState: State) {
        when (newState) {
            State.ON -> {
                knobView.translationX = knobMaxTranslation
                knobImage.paint.color = onColor
            }
            State.OFF -> {
                knobView.translationX = 0f
                knobImage.paint.color = offColor
            }
        }

        state = newState
    }

    private fun animateToState() {
        var playTime = (knobPosition * knobAnimationDuration).toLong()

        when (state) {
            State.ON -> {
                colorAnimation.start()
                positionAnimation.start()
            }
            State.OFF -> {
                colorAnimation.reverse()
                positionAnimation.reverse()

                playTime = knobAnimationDuration - playTime
            }
        }

        colorAnimation.currentPlayTime = playTime
        positionAnimation.currentPlayTime = playTime
    }
}
