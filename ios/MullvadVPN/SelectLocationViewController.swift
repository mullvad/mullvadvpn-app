//
//  SelectLocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import DiffableDataSources
import UIKit
import Logging

private let kCellIdentifier = "Cell"

class SelectLocationViewController: UITableViewController, RelayCacheObserver {

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

    private let logger = Logger(label: "SelectLocationController")
    private var cachedRelays: CachedRelays?
    private var relayConstraints: RelayConstraints?
    private var expandedItems = [RelayLocation]()
    private var dataSource: DataSource?

    var didSelectLocationHandler: ((RelayLocation) -> Void)?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        tableView.tableHeaderView = SelectLocationHeaderView(frame: CGRect(x: 0, y: 0, width: 50, height: 50))
        tableView.register(SelectLocationCell.self, forCellReuseIdentifier: kCellIdentifier)
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero

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

        dataSource?.defaultRowAnimation = .top
        tableView.dataSource = dataSource

        RelayCache.shared.addObserver(self)

        updateDataSource(animateDifferences: false) {
            self.updateTableViewSelection(scroll: true, animated: false)
        }
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

        // Disable interaction with the controller after selection
        tableView.isUserInteractionEnabled = false

        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(250)) {
            self.didSelectLocationHandler?(item.relayLocation)
        }
    }

    // MARK: - RelayCacheObserver

    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelays cachedRelays: CachedRelays) {
        self.didReceiveCachedRelays(cachedRelays) { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let (cachedRelays, relayConstraints)):
                    self.didReceiveCachedRelays(cachedRelays, relayConstraints: relayConstraints)

                case .failure(let error):
                    self.logger.error(chainedError: error)
                }
            }
        }
    }

    // MARK: - Public

    func prefetchData(completionHandler: @escaping () -> Void) {
        fetchRelays { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let (cachedRelays, relayConstraints)):
                    self.didReceiveCachedRelays(cachedRelays, relayConstraints: relayConstraints)

                case .failure(let error):
                    self.logger.error(chainedError: error)
                }

                completionHandler()
            }
        }
    }

    // MARK: - Relay list handling

    private func fetchRelays(completionHandler: @escaping (Result<(CachedRelays, RelayConstraints), Error>) -> Void) {
        RelayCache.shared.read { (result) in
            switch result {
            case .success(let cachedRelays):
                self.didReceiveCachedRelays(cachedRelays, completionHandler: completionHandler)

            case .failure(let error):
                completionHandler(.failure(.loadRelayList(error)))
            }
        }
    }

    private func didReceiveCachedRelays(_ cachedRelays: CachedRelays, completionHandler: @escaping (Result<(CachedRelays, RelayConstraints), Error>) -> Void) {
        TunnelManager.shared.getRelayConstraints { (result) in
            let result = result
                .map { (cachedRelays, $0) }
                .mapError { Error.getRelayConstraints($0) }

            completionHandler(result)
        }
    }

    private func didReceiveCachedRelays(_ cachedRelays: CachedRelays, relayConstraints: RelayConstraints) {
        self.cachedRelays = cachedRelays
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
        let items = self.cachedRelays.map { (cachedRelays) -> [DataSourceItem] in
            return cachedRelays.relays.makeDataSource { (item) -> Bool in
                return expandedItems.contains(item.relayLocation)
            }
        } ?? []

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
        let location: String
        let name: String
        let hasActiveRelays: Bool
    }

    struct City {
        let location: String
        let name: String
        let hasActiveRelays: Bool
    }

    struct Hostname {
        let location: String
        let hostname: String
        let active: Bool
    }

    case country(Country)
    case city(City)
    case hostname(Hostname)

    var relayLocation: RelayLocation {
        switch self {
        case .country(let country):
            return .country(country.location)
        case .city(let city):
            let split = city.location.split(separator: "-", maxSplits: 2).map(String.init)
            return .city(split[0], split[1])
        case .hostname(let host):
            let split = host.location.split(separator: "-", maxSplits: 2).map(String.init)
            return .hostname(split[0], split[1], host.hostname)
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

extension ServerRelaysResponse {
    fileprivate static func lexicalSortComparator(_ a: String, _ b: String) -> Bool {
        return a.localizedCaseInsensitiveCompare(b) == .orderedAscending
    }

    fileprivate static func fileSortComparator(_ a: String, _ b: String) -> Bool {
        return a.localizedStandardCompare(b) == .orderedAscending
    }

    fileprivate func makeDataSource(evaluator: (DataSourceItem) -> Bool) -> [DataSourceItem] {
        let relaysByCountry = Dictionary(grouping: wireguard.relays) { (relay) -> String in
            return relay.location.split(separator: "-").first.flatMap(String.init)!
        }

        var items = [DataSourceItem]()

        var countryItems = [DataSourceItem.Country]()
        var cityItems = [String: [DataSourceItem.City]]()
        var relayItems = [String: [DataSourceItem.Hostname]]()

        for (countryCode, relays) in relaysByCountry {
            let relaysByCity = Dictionary(grouping: relays) { (relay) -> String in
                return relay.location
            }

            if let (cityCode, relays) = relaysByCity.first {
                guard let location = locations[cityCode] else {
                    continue
                }

                let country = DataSourceItem.Country(
                    location: countryCode,
                    name: location.country,
                    hasActiveRelays: relays.contains(where: { (serverRelay) -> Bool in
                        return serverRelay.active
                    }))

                countryItems.append(country)
                if !evaluator(.country(country)) {
                    continue
                }
            }

            for (cityCode, relays) in relaysByCity {
                guard let location = locations[cityCode] else {
                    // TODO: log to file?
                    print("Location not found: \(cityCode)")
                    continue
                }

                let city = DataSourceItem.City(
                    location: cityCode,
                    name: location.city,
                    hasActiveRelays: relays.contains(where: { (serverRelay) -> Bool in
                        return serverRelay.active
                    }))

                if var cities = cityItems[countryCode] {
                    cities.append(city)
                    cityItems[countryCode] = cities
                } else {
                    cityItems[countryCode] = [city]
                }

                if !evaluator(.city(city)) {
                    continue
                }

                relayItems[cityCode] = relays.map { (relay) -> DataSourceItem.Hostname in
                    return DataSourceItem.Hostname(location: relay.location, hostname: relay.hostname, active: relay.active)
                }
            }
        }

        countryItems.sort { (a, b) -> Bool in
            return Self.lexicalSortComparator(a.name, b.name)
        }

        for country in countryItems {
            items.append(.country(country))

            if var cities = cityItems[country.location] {
                cities.sort { (a, b) -> Bool in
                    return Self.lexicalSortComparator(a.name, b.name)
                }
                for city in cities {
                    items.append(.city(city))

                    if var relays = relayItems[city.location] {
                        relays.sort { (a, b) -> Bool in
                            return Self.fileSortComparator(a.hostname, b.hostname)
                        }
                        items.append(contentsOf: relays.map { DataSourceItem.hostname($0) })
                    }
                }
            }
        }

        return items
    }
}
