package net.mullvad.mullvadvpn.model

import androidx.annotation.DrawableRes
import androidx.annotation.IntDef
import androidx.annotation.StringRes
import java.lang.IllegalArgumentException

class ListItemData private constructor(builder: Builder) {

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
        if (builder.identifier == null)
            throw IllegalArgumentException("ListItem must have identifier for proper animation")

        this.identifier = builder.identifier!!

        this.type = builder.type

        if ((builder.text == null && builder.textRes == null) && type > PROGRESS)
            throw IllegalArgumentException("ListItem should have configured with text")

        this.text = builder.text
        this.textRes = builder.textRes
        this.iconRes = builder.iconRes
        this.isSelected = builder.isSelected
        this.widget = builder.widget
        this.action = builder.action
    }

    override fun equals(other: Any?): Boolean {
        if (other !is ListItemData)
            return false

        if (other.identifier != this.identifier ||
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

    override fun hashCode(): Int {
        var result = identifier.hashCode()
        result = 31 * result + (text?.hashCode() ?: 0)
        result = 31 * result + (textRes ?: 0)
        result = 31 * result + (iconRes ?: 0)
        result = 31 * result + isSelected.hashCode()
        result = 31 * result + type
        result = 31 * result + (widget?.hashCode() ?: 0)
        result = 31 * result + (action?.hashCode() ?: 0)
        return result
    }

    class Builder {
        var identifier: String? = null
            private set

        var text: String? = null
            private set

        @StringRes
        var textRes: Int? = null
            private set

        @DrawableRes
        var iconRes: Int? = null
            private set
        var isSelected: Boolean = false
            private set

        @ItemType
        var type: Int = 0
            private set

        var widget: WidgetState? = null
            private set
        var action: ItemAction? = null
            private set

        fun setIdentifier(id: String): Builder = apply {
            this.identifier = id
        }

        fun setText(text: String): Builder = apply {
            this.text = text
        }

        fun setTextRes(@StringRes textRes: Int): Builder = apply {
            this.textRes = textRes
        }

        fun setIconRes(@DrawableRes iconRes: Int): Builder = apply {
            this.iconRes = iconRes
        }

        fun setSelected(isSelected: Boolean): Builder = apply {
            this.isSelected = isSelected
        }

        fun setType(@ItemType type: Int): Builder = apply {
            this.type = type
        }

        fun setWidget(widget: WidgetState): Builder = apply {
            this.widget = widget
        }

        fun setAction(action: ItemAction): Builder = apply {
            this.action = action
        }

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
