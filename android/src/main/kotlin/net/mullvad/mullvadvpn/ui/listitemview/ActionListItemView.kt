package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import android.content.res.Resources
import android.util.AttributeSet
import android.view.ViewGroup
import android.widget.ImageView
import android.widget.TextView
import androidx.core.view.isVisible
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.WidgetState

open class ActionListItemView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = R.attr.actionListItemViewStyle,
    defStyleRes: Int = 0
) : ListItemView(context, attrs, defStyleAttr, defStyleRes) {

    protected var widgetController: WidgetViewController<*>? = null
    protected val itemText: TextView = findViewById(R.id.itemText)
    protected val itemIcon: ImageView = findViewById(R.id.itemIcon)
    protected val widgetContainer: ViewGroup = findViewById(R.id.widgetContainer)

    protected val clickListener = OnClickListener {
        itemData.action?.let { _ ->
            listItemListener?.onItemAction(itemData)
        }
    }

    override val layoutRes: Int
        get() = R.layout.list_item_action

    override val heightRes: Int
        get() = R.dimen.cell_height

    override fun onUpdate() {
        updateImage()
        updateText()
        updateWidget()
        updateAction()
    }

    protected open fun updateImage() {
        try {
            itemData.iconRes?.let {
                itemIcon.isVisible = true
                itemIcon.setImageResource(it)
                return
            }
        } catch (ignore: Resources.NotFoundException) {
            itemIcon.isVisible = true
            itemIcon.setImageResource(R.drawable.ic_icons_missing)
            return
        }

        itemIcon.isVisible = false
        itemIcon.setImageDrawable(null)
    }

    protected open fun updateText() {
        itemData.textRes?.let {
            itemText.setText(it)
            return
        }
        itemData.text?.let {
            itemText.setText(it)
            return
        }
        itemText.text = ""
    }

    protected open fun updateAction() {
        if (itemData.action == null) {
            setOnClickListener(null)
            isClickable = false
            isFocusable = false
        } else {
            setOnClickListener(clickListener)
            isClickable = true
            isFocusable = true
        }
    }

    protected open fun updateWidget() {
        itemData.widget.let { state ->
            when (state) {
                is WidgetState.ImageState -> {
                    if (widgetController !is WidgetViewController.StandardController) {
                        widgetContainer.removeAllViews()
                        widgetContainer.isVisible = true
                        widgetController = WidgetViewController.StandardController(widgetContainer)
                    }
                    (widgetController as WidgetViewController.StandardController).updateState(state)
                }
                is WidgetState.SwitchState -> {
                    if (widgetController !is WidgetViewController.SwitchController) {
                        widgetContainer.removeAllViews()
                        widgetContainer.isVisible = true
                        widgetController = WidgetViewController.SwitchController(widgetContainer)
                    }
                    (widgetController as WidgetViewController.SwitchController).updateState(state)
                }
                null -> {
                    if (widgetController != null) {
                        widgetController = null
                        widgetContainer.removeAllViews()
                        widgetContainer.isVisible = false
                    }
                }
            }
        }
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        widgetContainer.requestLayout()
    }
}
