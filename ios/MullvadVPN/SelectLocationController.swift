//
//  SelectLocationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit
import ProcedureKit
import os.log

private let cellIdentifier = "Cell"

class SelectLocationController: UITableViewController {

    private var relayList: RelayList?
    private var expandedItems = [RelayListDataSourceItem]()
    private var displayedItems = [RelayListDataSourceItem]()

    private let procedureQueue = ProcedureQueue()

    var selectedItem: RelayListDataSourceItem?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        loadRelayList()
        updateTableHeaderViewSize(tableViewSize: tableView.frame.size)
    }

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        super.viewWillTransition(to: size, with: coordinator)

        updateTableHeaderViewSize(tableViewSize: size)
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

    // MARK: - UIScrollViewDelegate

    override func scrollViewDidScroll(_ scrollView: UIScrollView) {
        updateBarVisibility(threshold: 12)
    }

    // MARK: - Relay list handling

    private func loadRelayList() {
        let procedure = MullvadAPI.getRelayList()

        procedure.addDidFinishBlockObserver(synchronizedWith: DispatchQueue.main) { [weak self] (procedure, error) in
            guard let response = procedure.output.success else {
                os_log(.error, "Relay list network error: %{public}s", error?.localizedDescription ?? "(null)")
                return
            }

            self?.didReceiveRelayList(response)
        }

        procedureQueue.addOperation(procedure)
    }

    private func didReceiveRelayList(_ response: JsonRpcResponse<RelayList>) {
        do {
            relayList = try response.result.get()
        } catch {
            os_log(.error, "Relay list server error: %{public}s", error.localizedDescription)
        }

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

    // MARK: - Bar visibility

    private func updateBarVisibility(threshold: CGFloat) {
        guard let navigationBar = navigationController?.navigationBar as? CustomNavigationBar else {
            return
        }

        let shouldShowBar = tableView.contentOffset.y > (-tableView.adjustedContentInset.top + threshold)

        navigationBar.setBarVisible(shouldShowBar, animated: true)
    }

    // MARK: - UITableView header

    private func updateTableHeaderViewSize(tableViewSize: CGSize) {
        guard let header = tableView.tableHeaderView else { return }

        // layout the header view
        header.setNeedsLayout()
        header.layoutIfNeeded()

        // measure the view size
        let sizeConstraint = CGSize(
            width: tableViewSize.width,
            height: UIView.layoutFittingCompressedSize.height
        )
        header.frame.size = header.systemLayoutSizeFitting(sizeConstraint)

        // reset the header view to force UITableView layout pass
        tableView.tableHeaderView = header
    }
}

/// Private extension to convert a RelayList into a flat list of RelayListDataSourceItems
private extension RelayList {

    typealias FilterFunc = (RelayListDataSourceItem) -> Bool

    func intoRelayDataSourceItemList(filter: FilterFunc) -> [RelayListDataSourceItem] {
        var items = [RelayListDataSourceItem]()

        for country in countries {
            let countryItem = RelayListDataSourceItem.country(country)

            items.append(countryItem)

            guard filter(countryItem) else { continue }

            for city in country.cities {
                let cityItem = RelayListDataSourceItem.city(city)

                items.append(cityItem)

                guard filter(cityItem) else { continue }

                for host in city.relays {
                    items.append(.hostname(host))
                }
            }
        }

        return items
    }

}

/// A wrapper type for RelayList to be able to represent it as a flat list
enum RelayListDataSourceItem: Equatable {

    case country(RelayList.Country)
    case city(RelayList.City)
    case hostname(RelayList.Hostname)

    static func == (lhs: RelayListDataSourceItem, rhs: RelayListDataSourceItem) -> Bool {
        switch (lhs, rhs) {
        case (.country(let a), .country(let b)):
            return a.code == b.code

        case (.city(let a), .city(let b)):
            return a.code == b.code

        case (.hostname(let a), .hostname(let b)):
            return a.hostname == b.hostname

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
            return country.cities.count > 0
        case .city(let city):
            return city.relays.count > 0
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
