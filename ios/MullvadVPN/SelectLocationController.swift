//
//  SelectLocationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import UIKit
import os

private let kCellIdentifier = "Cell"

enum SelectLocationControllerError: Error {
    case loadRelayList(RelayCacheError)
    case getRelayConstraints(TunnelManagerError)
}

class SelectLocationController: UITableViewController {

    private let relayCache = try! RelayCache.withDefaultLocation().get()
    private var relayList: RelayList?
    private var relayConstraints: RelayConstraints?
    private var expandedItems = [RelayLocation]()
    private var dataSource: DataSource?
    private var loadDataSubscriber: AnyCancellable?

    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!

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

        addActivityIndicatorView()
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

    // MARK: - Relay list handling

    private func loadData() {
        loadDataSubscriber = relayCache.read()
            .mapError { SelectLocationControllerError.loadRelayList($0) }
            .map { $0.relayList.sorted() }
            .flatMap({ (filteredRelayList) in
                TunnelManager.shared.getRelayConstraints()
                    .mapError { SelectLocationControllerError.getRelayConstraints($0) }
                    .map { (filteredRelayList, $0) }
            })
            .receive(on: DispatchQueue.main)
            .handleEvents(receiveSubscription: { [weak self] _ in
                self?.activityIndicator.startAnimating()
            }, receiveCompletion: { [weak self] _ in
                self?.activityIndicator.stopAnimating()
            }, receiveCancel: { [weak self] () in
                self?.activityIndicator.stopAnimating()
            })
            .sink(receiveCompletion: { (completion) in
                if case .failure(let error) = completion {
                    os_log(.error, "Failed to load the SelectLocation controller: %{public}s", error.localizedDescription)
                }
            }) { [weak self] (result) in
                let (relayList, constraints) = result

                self?.didReceive(relayList: relayList, relayConstraints: constraints)
            }
    }

    private func didReceive(relayList: RelayList, relayConstraints: RelayConstraints) {
        self.relayList = relayList
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

    // MARK: - Activity indicator

    private func addActivityIndicatorView() {
        view.addSubview(activityIndicator)

        activityIndicator.translatesAutoresizingMaskIntoConstraints = false

        NSLayoutConstraint.activate([
            activityIndicator.widthAnchor.constraint(equalToConstant: 48),
            activityIndicator.heightAnchor.constraint(equalToConstant: 48),
            activityIndicator.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            activityIndicator.centerYAnchor.constraint(equalTo: view.centerYAnchor, constant: -60)
        ])
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
private typealias DataSource = UITableViewDiffableDataSource<DataSourceSection, DataSourceItem>

/// Data source snapshot type
private typealias DataSourceSnapshot = NSDiffableDataSourceSnapshot<DataSourceSection, DataSourceItem>

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
