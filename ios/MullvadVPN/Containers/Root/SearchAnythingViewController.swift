//
//  SearchAnythingViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-03-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit
import MullvadREST
import MullvadTypes

class SearchAnythingViewController: UIViewController {
    struct Item {
        let title: String
        let destination: Destination?
        let location: RelayLocation?
        let cell: Cell
    }

    enum Destination {
        case account, selectLocation, changelog, daita, multihop, settings, vpnSettings, problemReport, faq, apiAccess, copyAccountNumber
    }

    enum Cell: String, CaseIterable, CellIdentifierProtocol {
        case setting, location

        var cellClass: AnyClass {
            switch self {
            case .setting, .location: SettingsCell.self
            }
        }

        var type: String {
            switch self {
            case .setting: "Setting"
            case .location: "Location"
            }
        }
    }

    let searchBar = UISearchBar()
    let tableView = UITableView()

    var items = [Item]()
    var searchItems = [Item]()

    var didSearch: ((String) -> Void)?
    var didSelect: ((Item) -> Void)?

    init(relays: [RelayWithLocation<REST.ServerRelay>]) {
        var added = [String]()

        items.append(Item(title: "Select location", destination: .selectLocation, location: nil, cell: .setting))
        items.append(Item(title: "Settings", destination: .settings, location: nil, cell: .setting))
        items.append(Item(title: "VPN settings", destination: .vpnSettings, location: nil, cell: .setting))
        items.append(Item(title: "API access", destination: .apiAccess, location: nil, cell: .setting))
        items.append(Item(title: "DAITA", destination: .daita, location: nil, cell: .setting))
        items.append(Item(title: "Multihop", destination: .multihop, location: nil, cell: .setting))
        items.append(Item(title: "Problem report", destination: .problemReport, location: nil, cell: .setting))
        items.append(Item(title: "Changelog", destination: .changelog, location: nil, cell: .setting))
        items.append(Item(title: "FAQ", destination: .faq, location: nil, cell: .setting))
        items.append(Item(title: "Copy account number", destination: .copyAccountNumber, location: nil, cell: .setting))

        let cities: [Item] = relays.compactMap {
            if added.contains($0.serverLocation.city) { return nil }
            added.append($0.serverLocation.city)
            return Item(title: $0.serverLocation.city, destination: nil, location: RelayLocation(dashSeparatedString: "\($0.serverLocation.countryCode)-\($0.serverLocation.cityCode)"), cell: .location)
        }.sorted { $0.title < $1.title }

        let countries: [Item] = relays.compactMap {
            if added.contains($0.serverLocation.country) { return nil }
            added.append($0.serverLocation.country)
            return Item(title: $0.serverLocation.country, destination: nil, location: RelayLocation(dashSeparatedString: "\($0.serverLocation.countryCode)"), cell: .location)
        }.sorted { $0.title < $1.title }

        items.append(contentsOf: countries)
        items.append(contentsOf: cities)

        searchItems = items

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        tableView.backgroundColor = .clear

        tableView.dataSource = self
        tableView.delegate = self
        tableView.keyboardDismissMode = .onDrag
        tableView.registerReusableViews(from: Cell.self)

        setUpSearchBar()

        view.addConstrainedSubviews([searchBar, tableView]) {
            searchBar.pinEdgesToSuperview(.all().excluding(.bottom))
            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: searchBar.bottomAnchor)
        }

        searchBar.becomeFirstResponder()
    }

    private func setUpSearchBar() {
        searchBar.delegate = self
        searchBar.searchBarStyle = .minimal
        searchBar.layer.cornerRadius = 8
        searchBar.clipsToBounds = true
        searchBar.placeholder = NSLocalizedString(
            "SEARCHBAR_PLACEHOLDER",
            tableName: "SearchAnything",
            value: "Search for...",
            comment: ""
        )

        UITextField.SearchTextFieldAppearance.inactive.apply(to: searchBar)
    }
}

extension SearchAnythingViewController: UITableViewDataSource {
    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        searchItems.count
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let item = searchItems[indexPath.row]

        let cell = (tableView.dequeueReusableCell(withIdentifier: item.cell.rawValue, for: indexPath) as? SettingsCell) ?? SettingsCell()
        cell.titleLabel.text = item.title
        cell.detailTitleLabel.text = item.cell.type

        return cell
    }
}

extension SearchAnythingViewController: UITableViewDelegate {
    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let item = searchItems[indexPath.row]
        didSelect?(item)
    }
}

extension SearchAnythingViewController: UISearchBarDelegate {
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        searchItems = searchText.isEmpty ? items : items.filter { $0.title.fuzzyMatch(searchText) }
        tableView.reloadData()
    }

    func searchBarTextDidBeginEditing(_ searchBar: UISearchBar) {
        UITextField.SearchTextFieldAppearance.active.apply(to: searchBar)
    }

    func searchBarTextDidEndEditing(_ searchBar: UISearchBar) {
        UITextField.SearchTextFieldAppearance.inactive.apply(to: searchBar)
    }
}
