package net.mullvad.mullvadvpn.model

import androidx.annotation.DrawableRes
import androidx.annotation.IntDef
import androidx.annotation.StringRes
import java.lang.IllegalArgumentException

class ListItemData internal constructor(builder: Builder) {

    val identifier: String

    val text: String?

    @StringRes
    val textRes: Int?

    @DrawableRes
    val iconRes: Int?

    val isSelected: Boolean

    @ItemType
    val type: Int

    @Retention
    @IntDef(DIVIDER, PLAIN, ACTION)
    annotation class ItemType

    val widget: WidgetState?

    val action: ItemAction?

    init {
        if (builder.getIdentifier() == null)
            throw IllegalArgumentException("ListItem must have identifier for proper animation")

        this.identifier = builder.getIdentifier()!!

        this.type = builder.getType()

        if ((builder.getText() == null && builder.getTextRes() == null) && type > PROGRESS)
            throw IllegalArgumentException("ListItem should have configured with text")

        this.text = builder.getText()
        this.textRes = builder.getTextRes()
        this.iconRes = builder.getIconRes()
        this.isSelected = builder.isSelected()
        this.widget = builder.getWidget()
        this.action = builder.getAction()
    }

    override fun equals(other: Any?): Boolean {
        if (other !is ListItemData)
            return false

        if (other.identifier!= this.identifier ||
            other.type != this.type ||
            other.text != this.text ||
            other.iconRes != this.iconRes ||
            other.textRes != this.textRes ||
            other.action != this.action ||
            other.widget != this.widget
        )
            return false
        return true
    }

    class Builder {
        private var identifier: String? = null

        private var text: String? = null

        @StringRes
        private var textRes: Int? = null

        @DrawableRes
        private var iconRes: Int? = null
        private var isSelected: Boolean = false

        @ItemType
        private var type: Int = 0

        private var widget: WidgetState? = null
        private var action: ItemAction? = null

        fun setIdentifier(id: String): Builder {
            this.identifier = id
            return this
        }

        fun setText(text: String): Builder {
            this.text = text
            return this
        }

        fun setTextRes(@StringRes textRes: Int): Builder {
            this.textRes = textRes
            return this
        }

        fun setIconRes(@DrawableRes iconRes: Int): Builder {
            this.iconRes = iconRes
            return this
        }

        fun setSelected(isSelected: Boolean): Builder {
            this.isSelected = isSelected
            return this
        }

        fun setType(@ItemType type: Int): Builder {
            this.type = type
            return this
        }

        fun setWidget(widget: WidgetState): Builder {
            this.widget = widget
            return this
        }

        fun setAction(action: ItemAction): Builder {
            this.action = action
            return this
        }

        fun getIdentifier() = identifier

        fun getText() = text

        @StringRes
        fun getTextRes() = textRes

        @DrawableRes
        fun getIconRes() = iconRes
        fun isSelected() = isSelected

        @ItemType
        fun getType() = type

        fun getWidget() = widget

        fun getAction() = action

        fun build(): ListItemData = ListItemData(this)
    }

    data class ItemAction(val identifier: String)

    companion object {
        const val DIVIDER = 0
        const val PROGRESS = 1
        const val PLAIN = 2
        const val ACTION = 3
        const val DOUBLE_ACTION = 4
        const val APPLICATION = 5
    }
}
