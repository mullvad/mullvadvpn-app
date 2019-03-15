package net.mullvad.mullvadvpn.relaylist

import android.support.v7.widget.RecyclerView.ViewHolder
import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.R

class RelayItemHolder(private val view: View) : ViewHolder(view) {
    private val name: TextView = view.findViewById(R.id.name)

    private val countryColor = view.context.getColor(R.color.blue)
    private val cityColor = view.context.getColor(R.color.blue40)
    private val relayColor = view.context.getColor(R.color.blue20)

    private val countryPadding = view.resources.getDimensionPixelSize(R.dimen.country_row_padding)
    private val cityPadding = view.resources.getDimensionPixelSize(R.dimen.city_row_padding)
    private val relayPadding = view.resources.getDimensionPixelSize(R.dimen.relay_row_padding)

    var item: RelayItem? = null
        set(value) {
            field = value
            updateView()
        }

    private fun updateView() {
        val item = this.item

        if (item != null) {
            name.text = item.name

            when (item.type) {
                RelayItemType.Country -> setViewStyle(countryColor, countryPadding)
                RelayItemType.City -> setViewStyle(cityColor, cityPadding)
                RelayItemType.Relay -> setViewStyle(relayColor, relayPadding)
            }
        } else {
            name.text = ""
        }
    }

    private fun setViewStyle(backgroundColor: Int, padding: Int) {
        val paddingLeft = padding
        val paddingTop = view.paddingTop
        val paddingRight = view.paddingRight
        val paddingBottom = view.paddingBottom

        view.apply {
            setBackgroundColor(backgroundColor)
            setPadding(paddingLeft, paddingTop, paddingRight, paddingBottom)
        }
    }
}
