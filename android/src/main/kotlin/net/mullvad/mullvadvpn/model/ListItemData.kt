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
        var text: String? = null
        @StringRes
        var textRes: Int? = null
        @DrawableRes
        var iconRes: Int? = null
        var isSelected: Boolean = false
        @ItemType
        var type: Int = 0
        var widget: WidgetState? = null
        var action: ItemAction? = null

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
        fun build(setUp: Builder.() -> Unit): ListItemData = Builder().also(setUp).build()
    }
}
