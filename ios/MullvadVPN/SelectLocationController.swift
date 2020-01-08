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

private let cellIdentifier = "Cell"

enum SelectLocationControllerError: Error {
    case loadRelayList(RelayCacheError)
    case getRelayConstraints(TunnelManagerError)
}

class SelectLocationController: UITableViewController {

    private let relayCache = try! RelayCache.withDefaultLocation().get()
    private var relayList: RelayList?
    private var relayConstraints: RelayConstraints?
    private var expandedItems = [RelayLocation]()
    private var dataSource = [RelayListDataSourceItem]()

    private var loadDataSubscriber: AnyCancellable?

    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!

    var selectedItem: RelayListDataSourceItem?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        addActivityIndicatorView()
        loadData()
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
        return dataSource.count
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(
            withIdentifier: cellIdentifier, for: indexPath) as! SelectLocationCell

        let item = dataSource[indexPath.row]

        cell.isDisabled = !item.hasActiveRelays()
        cell.locationLabel.text = item.displayName()
        cell.statusIndicator.isActive = item.hasActiveRelays()
        cell.showsCollapseControl = item.isCollapsibleLevel()
        cell.isExpanded = expandedItems.contains(item.relayLocation)
        cell.didCollapseHandler = { [weak self] (cell) in
            self?.collapseCell(cell)
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        let item = dataSource[indexPath.row]

        return item.hasActiveRelays()
    }

    override func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        let item = dataSource[indexPath.row]

        return item.indentationLevel()
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        selectedItem = dataSource[indexPath.row]

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
            .handleEvents(receiveSubscription: { _ in
                self.activityIndicator.startAnimating()
            }, receiveCompletion: { _ in
                self.activityIndicator.stopAnimating()
            }, receiveCancel: {
                self.activityIndicator.stopAnimating()
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

        updateDataSource()
        tableView.reloadData()

        updateTableViewSelection(scroll: true, animated: false)
    }

    private func computeIndexPathForSelectedLocation(relayLocation: RelayLocation) -> IndexPath? {
        guard let row = dataSource.firstIndex(where: { $0.relayLocation == relayLocation })
            else {
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

    private func updateDataSource() {
        dataSource = relayList?.intoRelayDataSourceItemList(filter: { (item) -> Bool in
            return expandedItems.contains(item.relayLocation)
        }) ?? []
    }

    private func collapseCell(_ cell: SelectLocationCell) {
        guard let cellIndexPath = tableView.indexPath(for: cell) else {
            return
        }

        let item = dataSource[cellIndexPath.row]
        let itemLocation = item.relayLocation
        let numberOfItemsBefore = dataSource.count

        if let index = expandedItems.firstIndex(of: itemLocation) {
            expandedItems.remove(at: index)
            cell.isExpanded = false
        } else {
            expandedItems.append(itemLocation)
            cell.isExpanded = true
        }

        updateDataSource()

        let numberOfItemsAfter = dataSource.count - numberOfItemsBefore
        let indexPathsOfAffectedItems = cellIndexPath.subsequentIndexPaths(count: abs(numberOfItemsAfter))

        tableView.performBatchUpdates({
            if numberOfItemsAfter > 0 {
                tableView.insertRows(at: indexPathsOfAffectedItems, with: .automatic)
            } else {
                tableView.deleteRows(at: indexPathsOfAffectedItems, with: .automatic)
            }
        }) { _ in
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

/// Private extension to convert a RelayList into a flat list of RelayListDataSourceItems
private extension RelayList {

    typealias FilterFunc = (RelayListDataSourceItem) -> Bool

    func intoRelayDataSourceItemList(filter: FilterFunc) -> [RelayListDataSourceItem] {
        var items = [RelayListDataSourceItem]()

        for country in countries {
            let wrappedCountry = RelayListDataSourceItem.Country(
                countryCode: country.code,
                name: country.name,
                hasActiveRelays: country.cities.contains(where: { (city) -> Bool in
                    return city.relays.contains { (host) -> Bool in
                        return host.active
                    }
                })
            )
            let countryItem = RelayListDataSourceItem.country(wrappedCountry)

            items.append(countryItem)

            guard country.cities.contains(where: { !$0.relays.isEmpty }) &&
                filter(countryItem) else { continue }

            for city in country.cities {
                let wrappedCity = RelayListDataSourceItem.City(
                    countryCode: country.code,
                    cityCode: city.code,
                    name: city.name,
                    hasActiveRelays: city.relays.contains(where: { $0.active })
                )
                let cityItem = RelayListDataSourceItem.city(wrappedCity)

                items.append(cityItem)

                guard !city.relays.isEmpty && filter(cityItem) else { continue }

                for host in city.relays {
                    let wrappedHost = RelayListDataSourceItem.Hostname(
                        countryCode: country.code,
                        cityCode: city.code,
                        hostname: host.hostname,
                        active: host.active)
                    items.append(.hostname(wrappedHost))
                }
            }
        }

        return items
    }

}

/// A wrapper type for RelayList to be able to represent it as a flat list
enum RelayListDataSourceItem {

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
}

extension RelayListDataSourceItem {

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

private extension IndexPath {
    func subsequentIndexPaths(count: Int) -> [IndexPath] {
        return (1...count).map({ IndexPath(row: self.row + $0, section: self.section) })
    }
}
