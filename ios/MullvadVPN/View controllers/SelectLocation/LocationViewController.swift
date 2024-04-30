//
//  LocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

protocol LocationViewControllerDelegate: AnyObject {
    func didRequestRouteToCustomLists(_ controller: LocationViewController, nodes: [LocationNode])
}

final class LocationViewController: UIViewController {
    private let searchBar = UISearchBar()
    private let tableView = UITableView(frame: .zero, style: .grouped)
    private let topContentView = UIStackView()
    private let filterView = RelayFilterView()
    private var dataSource: LocationDataSource?
    private var cachedRelays: CachedRelays?
    private var filter = RelayFilter()
    var relayLocations: UserSelectedRelays?
    weak var delegate: LocationViewControllerDelegate?
    var customListRepository: CustomListRepositoryProtocol

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var filterViewShouldBeHidden: Bool {
        return (filter.ownership == .any) && (filter.providers == .any)
    }

    var navigateToFilter: (() -> Void)?
    var didSelectRelays: ((UserSelectedRelays) -> Void)?
    var didUpdateFilter: ((RelayFilter) -> Void)?
    var didFinish: (() -> Void)?

    init(customListRepository: CustomListRepositoryProtocol) {
        self.customListRepository = customListRepository
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

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "SelectLocation",
            value: "Select location",
            comment: ""
        )

        navigationItem.leftBarButtonItem = UIBarButtonItem(
            title: NSLocalizedString(
                "NAVIGATION_TITLE",
                tableName: "SelectLocation",
                value: "Filter",
                comment: ""
            ),
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.navigateToFilter?()
            })
        )
        navigationItem.leftBarButtonItem?.accessibilityIdentifier = .selectLocationFilterButton

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.didFinish?()
            })
        )
        navigationItem.rightBarButtonItem?.accessibilityIdentifier = .closeSelectLocationButton

        setUpDataSources()
        setUpTableView()
        setUpTopContent()

        view.addConstrainedSubviews([topContentView, tableView]) {
            topContentView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))

            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: topContentView.bottomAnchor)
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

        dataSource?.setRelays(cachedRelays.relays, selectedRelays: relayLocations, filter: filter)
    }

    func refreshCustomLists() {
        dataSource?.refreshCustomLists(selectedRelays: relayLocations)
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
            self?.didSelectRelays?(locations)
        }

        dataSource?.didTapEditCustomLists = { [weak self] in
            guard let self else { return }
            delegate?.didRequestRouteToCustomLists(self, nodes: allLocationDataSource.nodes)
        }

        if let cachedRelays {
            dataSource?.setRelays(cachedRelays.relays, selectedRelays: relayLocations, filter: filter)
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
            didUpdateFilter?($0)

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
