//
//  SelectLocationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import DiffableDataSources
import UIKit
import os

private let kCellIdentifier = "Cell"

class SelectLocationController: UITableViewController, RelayCacheObserver {

    private enum Error: ChainedError {
        case loadRelayList(RelayCacheError)
        case getRelayConstraints(TunnelManager.Error)

        var errorDescription: String? {
            switch self {
            case .loadRelayList:
                return "Failure to load a relay list"
            case .getRelayConstraints:
                return "Failure to get relay constraints"
            }
        }
    }

    private var relayList: RelayList?
    private var relayConstraints: RelayConstraints?
    private var expandedItems = [RelayLocation]()
    private var dataSource: DataSource?

    var selectedLocation: RelayLocation?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        dataSource = DataSource(
            tableView: self.tableView,
            cellProvider: { [weak self] (tableView, indexPath, item) -> UITableViewCell? in
                guard let self = self else { return nil }

                let cell = tableView.dequeueReusableCell(
                    withIdentifier: kCellIdentifier, for: indexPath) as! SelectLocationCell

                cell.accessibilityIdentifier = item.relayLocation.stringRepresentation
                cell.isDisabled = !item.hasActiveRelays()
                cell.locationLabel.text = item.displayName()
                cell.statusIndicator.isActive = item.hasActiveRelays()
                cell.showsCollapseControl = item.isCollapsibleLevel()
                cell.isExpanded = self.expandedItems.contains(item.relayLocation)
                cell.didCollapseHandler = { [weak self] (cell) in
                    self?.collapseCell(cell)
                }

                return cell
        })

        tableView.dataSource = dataSource

        RelayCache.shared.addObserver(self)
        loadData()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        updateTableHeaderViewSizeIfNeeded()
    }

    // MARK: - UITableViewDelegate

    override func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        return dataSource?.itemIdentifier(for: indexPath)?.hasActiveRelays() ?? false
    }

    override func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        return dataSource?.itemIdentifier(for: indexPath)?.indentationLevel() ?? 0
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = dataSource?.itemIdentifier(for: indexPath) else { return }

        selectedLocation = item.relayLocation

        // Return back to the main view after selecting the relay
        tableView.isUserInteractionEnabled = false

        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(250)) {
            self.performSegue(withIdentifier:
                SegueIdentifier.SelectLocation.returnToConnectWithNewRelay.rawValue, sender: self)
        }
    }

    // MARK: - RelayCacheObserver

    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelayList cachedRelayList: CachedRelayList) {
        self.didReceiveCachedRelays(cachedRelayList: cachedRelayList) { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let (cachedRelayList, relayConstraints)):
                    self.didReceive(relayList: cachedRelayList.relayList, relayConstraints: relayConstraints)

                case .failure(let error):
                    error.logChain()
                }
            }
        }
    }

    // MARK: - Relay list handling

    private func loadData() {
        fetchRelays { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let (cachedRelayList, relayConstraints)):
                    self.didReceive(relayList: cachedRelayList.relayList, relayConstraints: relayConstraints)

                case .failure(let error):
                    error.logChain()
                }
            }
        }
    }

    private func fetchRelays(completionHandler: @escaping (Result<(CachedRelayList, RelayConstraints), Error>) -> Void) {
        RelayCache.shared.read { (result) in
            switch result {
            case .success(let cachedRelayList):
                self.didReceiveCachedRelays(cachedRelayList: cachedRelayList, completionHandler: completionHandler)

            case .failure(let error):
                completionHandler(.failure(.loadRelayList(error)))
            }
        }
    }

    private func didReceiveCachedRelays(cachedRelayList: CachedRelayList, completionHandler: @escaping (Result<(CachedRelayList, RelayConstraints), Error>) -> Void) {
        TunnelManager.shared.getRelayConstraints { (result) in
            let result = result
                .map { (cachedRelayList, $0) }
                .mapError { Error.getRelayConstraints($0) }

            completionHandler(result)
        }
    }

    private func didReceive(relayList: RelayList, relayConstraints: RelayConstraints) {
        self.relayList = relayList.sorted()
        self.relayConstraints = relayConstraints

        let relayLocation = relayConstraints.location.value
        expandedItems = relayLocation?.ascendants ?? []

        updateDataSource(animateDifferences: false)
        tableView.reloadData()

        updateTableViewSelection(scroll: true, animated: false)
    }

    private func computeIndexPathForSelectedLocation(relayLocation: RelayLocation) -> IndexPath? {
        guard let row = dataSource?.snapshot()
            .itemIdentifiers
            .firstIndex(where: { $0.relayLocation == relayLocation }) else {
                return nil
        }

        return IndexPath(row: row, section: 0)
    }

    // MARK: - Collapsible cells

    private func updateTableViewSelection(scroll: Bool, animated: Bool) {
        guard let relayLocation = relayConstraints?.location.value else { return }

        let indexPath = computeIndexPathForSelectedLocation(relayLocation: relayLocation)

        let scrollPosition: UITableView.ScrollPosition = scroll ? .middle : .none
        tableView.selectRow(at: indexPath, animated: animated, scrollPosition: scrollPosition)
    }

    private func updateDataSource(animateDifferences: Bool, completion: (() -> Void)? = nil) {
        let items = relayList?.intoRelayDataSourceItemList(using: { (item) -> Bool in
            return expandedItems.contains(item.relayLocation)
        }) ?? []

        var snapshot = DataSourceSnapshot()
        snapshot.appendSections([.locations])
        snapshot.appendItems(items, toSection: .locations)

        dataSource?.apply(
            snapshot,
            animatingDifferences: animateDifferences,
            completion: completion
        )
    }

    private func collapseCell(_ cell: SelectLocationCell) {
        guard let cellIndexPath = tableView.indexPath(for: cell),
            let item = dataSource?.itemIdentifier(for: cellIndexPath) else {
                return
        }

        let itemLocation = item.relayLocation

        if let index = expandedItems.firstIndex(of: itemLocation) {
            expandedItems.remove(at: index)
            cell.isExpanded = false
        } else {
            expandedItems.append(itemLocation)
            cell.isExpanded = true
        }

        updateDataSource(animateDifferences: true) {
            self.updateTableViewSelection(scroll: false, animated: true)
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

private extension RelayList {

    typealias EvaluatorFn = (DataSourceItem) -> Bool

    /// Turn `RelayList` into a flat list of `DataSourceItem`s.
    ///
    /// - Parameters evaluator: A closure that determines if the sub-tree should be rendered when it
    ///                         returns `true`, or dropped when it returns `false`
    func intoRelayDataSourceItemList(using evaluator: EvaluatorFn) -> [DataSourceItem] {
        var items = [DataSourceItem]()

        for country in countries {
            let wrappedCountry = DataSourceItem.Country(
                countryCode: country.code,
                name: country.name,
                hasActiveRelays: country.cities.contains(where: { (city) -> Bool in
                    return city.relays.contains { (host) -> Bool in
                        return host.active
                    }
                })
            )

            let countryItem = DataSourceItem.country(wrappedCountry)
            items.append(countryItem)

            if evaluator(countryItem) {
                for city in country.cities {
                    let wrappedCity = DataSourceItem.City(
                        countryCode: country.code,
                        cityCode: city.code,
                        name: city.name,
                        hasActiveRelays: city.relays.contains(where: { $0.active })
                    )

                    let cityItem = DataSourceItem.city(wrappedCity)
                    items.append(cityItem)

                    if evaluator(cityItem) {
                        for host in city.relays {
                            let wrappedHost = DataSourceItem.Hostname(
                                countryCode: country.code,
                                cityCode: city.code,
                                hostname: host.hostname,
                                active: host.active)
                            items.append(.hostname(wrappedHost))
                        }
                    }
                }
            }
        }

        return items
    }

}

private extension RelayLocation {

    /// A list of `RelayLocation` items preceding the given one in the relay tree
    var ascendants: [RelayLocation] {
        switch self {
        case .hostname(let country, let city, _):
            return [.country(country), .city(country, city)]

        case .city(let country, _):
            return [.country(country)]

        case .country:
            return []
        }
    }

}

/// Enum describing the table view sections
private enum DataSourceSection {
    case locations
}

/// Data source type
private typealias DataSource = TableViewDiffableDataSource<DataSourceSection, DataSourceItem>

/// Data source snapshot type
private typealias DataSourceSnapshot = DiffableDataSourceSnapshot<DataSourceSection, DataSourceItem>

/// A wrapper type for RelayList to be able to represent it as a flat list
private enum DataSourceItem: Hashable {

    struct Country {
        let countryCode: String
        let name: String
        let hasActiveRelays: Bool
    }

    struct City {
        let countryCode: String
        let cityCode: String
        let name: String
        let hasActiveRelays: Bool
    }

    struct Hostname {
        let countryCode: String
        let cityCode: String
        let hostname: String
        let active: Bool
    }

    case country(Country)
    case city(City)
    case hostname(Hostname)

    var relayLocation: RelayLocation {
        switch self {
        case .country(let country):
            return .country(country.countryCode)
        case .city(let city):
            return .city(city.countryCode, city.cityCode)
        case .hostname(let host):
            return .hostname(host.countryCode, host.cityCode, host.hostname)
        }
    }

    static func == (lhs: DataSourceItem, rhs: DataSourceItem) -> Bool {
        lhs.relayLocation == rhs.relayLocation
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(relayLocation)
    }

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
            return country.hasActiveRelays
        case .city(let city):
            return city.hasActiveRelays
        case .hostname(let host):
            return host.active
        }
    }

    func isCollapsibleLevel() -> Bool {
        switch self {
        case .country, .city:
            return self.hasActiveRelays()
        case .hostname:
            return false
        }
    }

}
