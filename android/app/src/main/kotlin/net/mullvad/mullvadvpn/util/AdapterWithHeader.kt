package net.mullvad.mullvadvpn.util

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.recyclerview.widget.RecyclerView
import androidx.recyclerview.widget.RecyclerView.ViewHolder
import kotlin.properties.Delegates.observable

class AdapterWithHeader<H : ViewHolder>(
    val adapter: RecyclerView.Adapter<H>,
    val headerLayoutId: Int
) : RecyclerView.Adapter<HeaderOrHolder<H>>() {
    private val observer =
        object : RecyclerView.AdapterDataObserver() {
            override fun onChanged() {
                notifyDataSetChanged()
            }

            override fun onItemRangeChanged(start: Int, count: Int) {
                notifyItemRangeChanged(start + 1, count)
            }

            override fun onItemRangeChanged(start: Int, count: Int, payload: Any?) {
                notifyItemRangeChanged(start + 1, count, payload)
            }

            override fun onItemRangeInserted(start: Int, count: Int) {
                notifyItemRangeInserted(start + 1, count)
            }

            override fun onItemRangeMoved(from: Int, to: Int, count: Int) {
                if (from == to) {
                    notifyItemRangeChanged(from + 1, count)
                } else {
                    val sourceStart = from + 1
                    val sourceEnd = sourceStart + count
                    val destinationStart = to + 1
                    val destinationEnd = destinationStart + count

                    val ascendingIndices =
                        (sourceStart..sourceEnd).zip(destinationStart..destinationEnd)

                    val indices =
                        if (from < to) {
                            ascendingIndices.asReversed()
                        } else {
                            ascendingIndices
                        }

                    for ((source, destination) in indices) {
                        notifyItemMoved(source, destination)
                    }
                }
            }

            override fun onItemRangeRemoved(start: Int, count: Int) {
                notifyItemRangeRemoved(start + 1, count)
            }
        }

    private var headerView: View? by
        observable<View?>(null) { _, _, newView ->
            newView?.let { view -> onHeaderAvailable?.invoke(view) }
        }

    var onHeaderAvailable by
        observable<((View) -> Unit)?>(null) { _, _, listener ->
            headerView?.let { header -> listener?.invoke(header) }
        }

    init {
        adapter.registerAdapterDataObserver(observer)
    }

    override fun getItemCount() = adapter.itemCount + 1

    override fun getItemId(position: Int): Long {
        if (position == 0) {
            return 0L
        } else {
            return adapter.getItemId(position - 1) + 1
        }
    }

    override fun getItemViewType(position: Int): Int {
        if (position == 0) {
            return 0
        } else {
            return adapter.getItemViewType(position - 1) + 1
        }
    }

    override fun onBindViewHolder(holder: HeaderOrHolder<H>, position: Int) {
        when (holder) {
            is HeaderOrHolder.Header -> {
                if (position != 0) {
                    throw IllegalArgumentException("Adapter position is not for the header")
                }
            }
            is HeaderOrHolder.Holder -> {
                if (position > 0) {
                    adapter.onBindViewHolder(holder.holder, position - 1)
                } else {
                    throw IllegalArgumentException("Adapter position is for the header")
                }
            }
        }
    }

    override fun onCreateViewHolder(parentView: ViewGroup, viewType: Int): HeaderOrHolder<H> {
        if (viewType == 0) {
            val inflater = LayoutInflater.from(parentView.context)
            val view = inflater.inflate(headerLayoutId, parentView, false)

            headerView = view

            return HeaderOrHolder.Header(view)
        } else {
            val holder = adapter.onCreateViewHolder(parentView, viewType - 1)

            return HeaderOrHolder.Holder(holder)
        }
    }
}
