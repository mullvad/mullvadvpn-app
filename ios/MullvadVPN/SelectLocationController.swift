//
//  SelectLocationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit
import os

private let cellIdentifier = "Cell"

class SelectLocationController: UITableViewController {

    private let relayCache = try! RelayCache.withDefaultLocation()
    private var relayList: RelayList?
    private var expandedItems = [RelayListDataSourceItem]()
    private var displayedItems = [RelayListDataSourceItem]()

    var selectedItem: RelayListDataSourceItem?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        loadRelayList()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        updateTableHeaderViewSizeIfNeeded()
    }

    // MARK: - UITableViewDataSource

    override func numberOfSections(in tableView: UITableView) -> Int {
        return 1
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        return displayedItems.count
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: cellIdentifier, for: indexPath) as! SelectLocationCell
        let item = displayedItems[indexPath.row]

        cell.locationLabel.text = item.displayName()
        cell.statusIndicator.isActive = item.hasActiveRelays()
        cell.showsCollapseControl = item.isCollapsibleLevel()
        cell.isExpanded = expandedItems.contains(item)
        cell.didCollapseHandler = { [weak self] (cell) in
            self?.collapseCell(cell)
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        let item = displayedItems[indexPath.row]

        return item.indentationLevel()
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        selectedItem = displayedItems[indexPath.row]

        // Return back to the main view after selecting the relay
        tableView.isUserInteractionEnabled = false
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(250)) {
            self.performSegue(withIdentifier:
                SegueIdentifier.SelectLocation.returnToConnectWithNewRelay.rawValue, sender: self)
        }
    }

    // MARK: - Relay list handling

    private func loadRelayList() {
        relayCache.read { [weak self] (result) in
            switch result {
            case .success(let cachedRelays):
                DispatchQueue.main.async {
                    self?.didReceiveRelayList(cachedRelays.relayList)
                }

            case .failure(let error):
                os_log(.error, "Failed to read the relay cache: %{public}s", error.localizedDescription)
            }
        }
    }

    private func didReceiveRelayList(_ relayList: RelayList) {
        self.relayList = relayList

        updateDisplayedItems()
        tableView.reloadData()
    }

    // MARK: - Collapsible cells

    private func updateDisplayedItems() {
        displayedItems = relayList?.intoRelayDataSourceItemList(filter: { expandedItems.contains($0)  }) ?? []
    }

    private func collapseCell(_ cell: SelectLocationCell) {
        guard let cellIndexPath = tableView.indexPath(for: cell) else {
            return
        }

        let item = displayedItems[cellIndexPath.row]

        let numberOfItemsBefore = displayedItems.count

        if let index = expandedItems.firstIndex(of: item) {
            expandedItems.remove(at: index)
            cell.isExpanded = false
        } else {
            expandedItems.append(item)
            cell.isExpanded = true
        }

        updateDisplayedItems()

        let numberOfItemsAfter = displayedItems.count - numberOfItemsBefore
        let indexPathsOfAffectedItems = cellIndexPath.subsequentIndexPaths(count: abs(numberOfItemsAfter))

        if numberOfItemsAfter > 0 {
            tableView.insertRows(at: indexPathsOfAffectedItems, with: .automatic)
        } else {
            tableView.deleteRows(at: indexPathsOfAffectedItems, with: .automatic)
        }
    }

    // MARK: - UITableView header

    private func updateTableHeaderViewSizeIfNeeded() {
        guard let header = tableView.tableHeaderView else { return }

        // measure the view size
        let sizeConstraint = CGSize(
            width: tableView.bounds.width,
            height: UIView.layoutFittingCompressedSize.height
        )

        let newSize = header.systemLayoutSizeFitting(sizeConstraint)
        let oldSize = header.frame.size

        if oldSize.height != newSize.height {
            header.frame.size.height = newSize.height

            // reset the header view to force UITableView layout pass
            tableView.tableHeaderView = header
        }
    }
}

/// Private extension to convert a RelayList into a flat list of RelayListDataSourceItems
private extension RelayList {

    typealias FilterFunc = (RelayListDataSourceItem) -> Bool

    func intoRelayDataSourceItemList(filter: FilterFunc) -> [RelayListDataSourceItem] {
        var items = [RelayListDataSourceItem]()

        for country in countries {
            let wrappedCountry = RelayListDataSourceItem.Country(
                countryCode: country.code,
                name: country.name,
                cityCount: country.cities.count)
            let countryItem = RelayListDataSourceItem.country(wrappedCountry)

            items.append(countryItem)

            guard filter(countryItem) else { continue }

            for city in country.cities {
                let wrappedCity = RelayListDataSourceItem.City(
                    countryCode: country.code,
                    cityCode: city.code,
                    name: city.name,
                    hostCount: city.relays.count)
                let cityItem = RelayListDataSourceItem.city(wrappedCity)

                items.append(cityItem)

                guard filter(cityItem) else { continue }

                for host in city.relays {
                    let wrappedHost = RelayListDataSourceItem.Hostname(
                        countryCode: country.code,
                        cityCode: city.code,
                        hostname: host.hostname)
                    items.append(.hostname(wrappedHost))
                }
            }
        }

        return items
    }

}

/// A wrapper type for RelayList to be able to represent it as a flat list
enum RelayListDataSourceItem: Equatable {

    struct Country {
        let countryCode: String
        let name: String
        let cityCount: Int
    }

    struct City {
        let countryCode: String
        let cityCode: String
        let name: String
        let hostCount: Int
    }

    struct Hostname {
        let countryCode: String
        let cityCode: String
        let hostname: String
    }

    case country(Country)
    case city(City)
    case hostname(Hostname)

    static func == (lhs: RelayListDataSourceItem, rhs: RelayListDataSourceItem) -> Bool {
        switch (lhs, rhs) {
        case (.country(let a), .country(let b)):
            return a.countryCode == b.countryCode

        case (.city(let a), .city(let b)):
            return a.countryCode == b.countryCode && a.cityCode == b.cityCode

        case (.hostname(let a), .hostname(let b)):
            return a.countryCode == b.countryCode && a.cityCode == b.cityCode &&
                a.hostname == b.hostname

        default:
            return false
        }
    }

}

private extension RelayListDataSourceItem {

    func indentationLevel() -> Int {
        switch self {
        case .country:
            return 0
        case .city:
            return 1
        case .hostname:
            return 2
        }
    }

    func displayName() -> String {
        switch self {
        case .country(let country):
            return country.name
        case .city(let city):
            return city.name
        case .hostname(let relay):
            return relay.hostname
        }
    }

    func hasActiveRelays() -> Bool {
        switch self {
        case .country(let country):
            return country.cityCount > 0
        case .city(let city):
            return city.hostCount > 0
        case .hostname:
            return true
        }
    }

    func isCollapsibleLevel() -> Bool {
        switch self {
        case .country, .city:
            return true
        case .hostname:
            return false
        }
    }

}

private extension IndexPath {
    func subsequentIndexPaths(count: Int) -> [IndexPath] {
        return (1...count).map({ IndexPath(row: self.row + $0, section: self.section) })
    }
}
