//
//  LocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

protocol LocationViewControllerDelegate: AnyObject {
    func navigateToCustomLists(nodes: [LocationNode])
    func didSelectRelays(relays: UserSelectedRelays)
    func didUpdateFilter(filter: RelayFilter)
}

final class LocationViewController: UIViewController {
    private let searchBar = UISearchBar()
    private let tableView = UITableView(frame: .zero, style: .grouped)
    private let topContentView = UIStackView()
    private let filterView = RelayFilterView()
    private var dataSource: LocationDataSource?
    private var cachedRelays: CachedRelays?
    private var filter = RelayFilter()
    private var selectedRelays: UserSelectedRelays?
    weak var delegate: LocationViewControllerDelegate?
    var customListRepository: CustomListRepositoryProtocol

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var filterViewShouldBeHidden: Bool {
        return (filter.ownership == .any) && (filter.providers == .any)
    }

    init(customListRepository: CustomListRepositoryProtocol, selectedRelays: UserSelectedRelays?) {
        self.customListRepository = customListRepository
        self.selectedRelays = selectedRelays
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        view.accessibilityIdentifier = .selectLocationView
        view.backgroundColor = .secondaryColor

        setUpDataSources()
        setUpTableView()
        setUpTopContent()

        view.addConstrainedSubviews([topContentView, tableView]) {
            topContentView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))

            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: topContentView.bottomAnchor, constant: 8)
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        tableView.flashScrollIndicators()
    }

    // MARK: - Public

    func setCachedRelays(_ cachedRelays: CachedRelays, filter: RelayFilter) {
        self.cachedRelays = cachedRelays
        self.filter = filter

        if filterViewShouldBeHidden {
            filterView.isHidden = true
        } else {
            filterView.isHidden = false
            filterView.setFilter(filter)
        }

        dataSource?.setRelays(cachedRelays.relays, selectedRelays: selectedRelays, filter: filter)
    }

    func refreshCustomLists() {
        dataSource?.refreshCustomLists(selectedRelays: selectedRelays)
    }

    // MARK: - Private

    private func setUpDataSources() {
        let allLocationDataSource = AllLocationDataSource()
        let customListsDataSource = CustomListsDataSource(repository: customListRepository)

        dataSource = LocationDataSource(
            tableView: tableView,
            allLocations: allLocationDataSource,
            customLists: customListsDataSource
        )

        dataSource?.didSelectRelayLocations = { [weak self] locations in
            self?.delegate?.didSelectRelays(relays: locations)
        }

        dataSource?.didTapEditCustomLists = { [weak self] in
            self?.delegate?.navigateToCustomLists(nodes: allLocationDataSource.nodes)
        }

        if let cachedRelays {
            dataSource?.setRelays(cachedRelays.relays, selectedRelays: selectedRelays, filter: filter)
        }
    }

    private func setUpTableView() {
        tableView.backgroundColor = view.backgroundColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.rowHeight = 56
        tableView.sectionHeaderHeight = 56
        tableView.indicatorStyle = .white
        tableView.keyboardDismissMode = .onDrag
        tableView.accessibilityIdentifier = .selectLocationTableView
    }

    private func setUpTopContent() {
        topContentView.axis = .vertical
        topContentView.addArrangedSubview(filterView)
        topContentView.addArrangedSubview(searchBar)

        filterView.isHidden = filterViewShouldBeHidden

        filterView.didUpdateFilter = { [weak self] in
            guard let self else { return }

            filter = $0
            delegate?.didUpdateFilter(filter: $0)

            if let cachedRelays {
                setCachedRelays(cachedRelays, filter: filter)
            }
        }

        setUpSearchBar()
    }

    private func setUpSearchBar() {
        searchBar.delegate = self
        searchBar.searchBarStyle = .minimal
        searchBar.layer.cornerRadius = 8
        searchBar.clipsToBounds = true
        searchBar.placeholder = NSLocalizedString(
            "SEARCHBAR_PLACEHOLDER",
            tableName: "SelectLocation",
            value: "Search for...",
            comment: ""
        )
        searchBar.searchTextField.accessibilityIdentifier = .selectLocationSearchTextField

        UITextField.SearchTextFieldAppearance.inactive.apply(to: searchBar)
    }
}

extension LocationViewController: UISearchBarDelegate {
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        dataSource?.filterRelays(by: searchText)
    }

    func searchBarTextDidBeginEditing(_ searchBar: UISearchBar) {
        UITextField.SearchTextFieldAppearance.active.apply(to: searchBar)
    }

    func searchBarTextDidEndEditing(_ searchBar: UISearchBar) {
        UITextField.SearchTextFieldAppearance.inactive.apply(to: searchBar)
    }
}
