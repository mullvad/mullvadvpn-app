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
    private var selectedRelays: RelaySelection
    weak var delegate: LocationViewControllerDelegate?
    var customListRepository: CustomListRepositoryProtocol

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var filterViewShouldBeHidden: Bool {
        return (filter.ownership == .any) && (filter.providers == .any)
    }

    init(customListRepository: CustomListRepositoryProtocol, selectedRelays: RelaySelection) {
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

        setUpDataSource()
        setUpTableView()
        setUpTopContent()

        view.addConstrainedSubviews([topContentView, tableView]) {
            topContentView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))

            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: topContentView.bottomAnchor, constant: 8)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        dataSource?.scrollToSelectedRelay()
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

    func setSelectedRelays(_ selectedRelays: RelaySelection) {
        self.selectedRelays = selectedRelays
        dataSource?.setSelectedRelays(selectedRelays)
    }

    // MARK: - Private

    private func setUpDataSource() {
        dataSource = LocationDataSource(
            tableView: tableView,
            allLocations: AllLocationDataSource(),
            customLists: CustomListsDataSource(repository: customListRepository)
        )

        dataSource?.didSelectRelayLocations = { [weak self] relays in
            self?.delegate?.didSelectRelays(relays: relays)
        }

        dataSource?.didTapEditCustomLists = { [weak self] in
            guard let self else { return }

            if let cachedRelays {
                let allLocationDataSource = AllLocationDataSource()
                allLocationDataSource.reload(cachedRelays.relays, relays: cachedRelays.relays.wireguard.relays)
                delegate?.navigateToCustomLists(nodes: allLocationDataSource.nodes)
            }
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
