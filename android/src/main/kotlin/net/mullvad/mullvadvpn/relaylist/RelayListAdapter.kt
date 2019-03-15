package net.mullvad.mullvadvpn.relaylist

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup

import net.mullvad.mullvadvpn.R

class RelayListAdapter : Adapter<RelayItemHolder>() {
    private val relayList = fakeRelayList

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): RelayItemHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.relay_list_item, parentView, false)

        return RelayItemHolder(view)
    }

    override fun onBindViewHolder(holder: RelayItemHolder, position: Int) {
        var remaining = position

        for (country in relayList) {
            val itemOrCount = country.getItem(remaining)

            when (itemOrCount) {
                is GetItemResult.Item -> {
                    holder.item = itemOrCount.item
                    return
                }
                is GetItemResult.Count -> remaining -= itemOrCount.count
            }
        }
    }

    override fun getItemCount(): Int {
        return relayList.map { country -> country.getItemCount() }.sum()
    }
}

val fakeRelayList = listOf(
    RelayCountry(
        "Australia",
        listOf(
            RelayCity(
                "Brisbane",
                listOf(Relay("au-bne-001")),
                false
            ),
            RelayCity(
                "Melbourne",
                listOf(Relay("au-mel-002"), Relay("au-mel-003"), Relay("au-mel-004")),
                false
            ),
            RelayCity(
                "Perth",
                listOf(Relay("au-per-001")),
                false
            ),
            RelayCity(
                "Sydney",
                listOf(
                    Relay("au1-wireguard"),
                    Relay("au-syd-001"),
                    Relay("au-syd-002"),
                    Relay("au-mel-003")
                ),
                false
            )
        ),
        false
    ),
    RelayCountry(
        "South Africa",
        listOf(
            RelayCity(
                "Johannesburg",
                listOf(Relay("za-jnb-001")),
                false
            )
        ),
        false
    ),
    RelayCountry(
        "Sweden",
        listOf(
            RelayCity(
                "Gothenburg",
                listOf(
                    Relay("se3-wireguard"),
                    Relay("se5-wireguard"),
                    Relay("se-got-001"),
                    Relay("se-got-002"),
                    Relay("se-got-003"),
                    Relay("se-got-004"),
                    Relay("se-got-005"),
                    Relay("se-got-006"),
                    Relay("se-got-007")
                ),
                false
            ),
            RelayCity(
                "Helsingborg",
                listOf(
                    Relay("se-hel-001"),
                    Relay("se-hel-002"),
                    Relay("se-hel-003"),
                    Relay("se-hel-004"),
                    Relay("se-hel-007"),
                    Relay("se-hel-008")
                ),
                false
            ),
            RelayCity(
                "Malmö",
                listOf(
                    Relay("se4-wireguard"),
                    Relay("se-mma-001"),
                    Relay("se-mma-002"),
                    Relay("se-mma-003"),
                    Relay("se-mma-004"),
                    Relay("se-mma-005"),
                    Relay("se-mma-006"),
                    Relay("se-mma-007"),
                    Relay("se-mma-008"),
                    Relay("se-mma-009"),
                    Relay("se-mma-010")
                ),
                false
            ),
            RelayCity(
                "Stockholm",
                listOf(
                    Relay("se2-wireguard"),
                    Relay("se6-wireguard"),
                    Relay("se7-wireguard"),
                    Relay("se8-wireguard"),
                    Relay("se-sto-001"),
                    Relay("se-sto-002"),
                    Relay("se-sto-003"),
                    Relay("se-sto-004"),
                    Relay("se-sto-005"),
                    Relay("se-sto-006"),
                    Relay("se-sto-007"),
                    Relay("se-sto-008"),
                    Relay("se-sto-009"),
                    Relay("se-sto-010"),
                    Relay("se-sto-011"),
                    Relay("se-sto-012"),
                    Relay("se-sto-013"),
                    Relay("se-sto-014"),
                    Relay("se-sto-015"),
                    Relay("se-sto-016"),
                    Relay("se-sto-017"),
                    Relay("se-sto-018"),
                    Relay("se-sto-019"),
                    Relay("se-sto-020"),
                    Relay("se-sto-021"),
                    Relay("se-sto-022"),
                    Relay("se-sto-023"),
                    Relay("se-sto-024"),
                    Relay("se-sto-025")
                ),
                false
            )
        ),
        false
    )
)
