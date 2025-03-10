//
//  LocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

protocol LocationViewControllerDelegate: AnyObject {
    func navigateToCustomLists(nodes: [LocationNode])
    func navigateToDaitaSettings()
    func didSelectRelays(relays: UserSelectedRelays)
    func didUpdateFilter(filter: RelayFilter)
}

final class LocationViewController: UIViewController {
    private let searchBar = UISearchBar()
    private let tableView = UITableView(frame: .zero, style: .grouped)
    private let topContentView = UIStackView()
    private let filterView = RelayFilterView()
    private var daitaInfoView: UIView?
    private var dataSource: LocationDataSource?
    private var relaysWithLocation: LocationRelays?
    private var filter = RelayFilter()
    private var selectedRelays: RelaySelection
    private var shouldFilterDaita: Bool
    weak var delegate: LocationViewControllerDelegate?
    var customListRepository: CustomListRepositoryProtocol

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    init(
        customListRepository: CustomListRepositoryProtocol,
        selectedRelays: RelaySelection,
        shouldFilterDaita: Bool
    ) {
        self.customListRepository = customListRepository
        self.selectedRelays = selectedRelays
        self.shouldFilterDaita = shouldFilterDaita

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        view.setAccessibilityIdentifier(.selectLocationView)
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

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        tableView.flashScrollIndicators()
    }

    // MARK: - Public

    func setRelaysWithLocation(_ relaysWithLocation: LocationRelays, filter: RelayFilter) {
        self.relaysWithLocation = relaysWithLocation
        self.filter = filter
        filterView.setFilter(filter)
        dataSource?.setRelays(relaysWithLocation, selectedRelays: selectedRelays)
    }

    func setDaitaChip(_ isEnabled: Bool) {
        self.shouldFilterDaita = isEnabled
        filterView.setDaita(isEnabled)
    }

    func refreshCustomLists() {
        dataSource?.refreshCustomLists()
    }

    func setSelectedRelays(_ selectedRelays: RelaySelection) {
        self.selectedRelays = selectedRelays
        dataSource?.setSelectedRelays(selectedRelays)
    }

    func toggleDaitaAutomaticRouting(isEnabled: Bool) {
        guard isEnabled else {
            daitaInfoView?.removeFromSuperview()
            daitaInfoView = nil

            searchBar.searchTextField.isEnabled = true
            UITextField.SearchTextFieldAppearance.inactive.apply(to: searchBar)
            return
        }

        guard daitaInfoView == nil else { return }

        let daitaInfoView = DAITAInfoView()
        self.daitaInfoView = daitaInfoView

        daitaInfoView.didPressDaitaSettingsButton = { [weak self] in
            self?.delegate?.navigateToDaitaSettings()
        }

        view.addConstrainedSubviews([daitaInfoView]) {
            daitaInfoView.pinEdgesToSuperview(.all().excluding(.top))
            daitaInfoView.topAnchor.constraint(equalTo: tableView.topAnchor)
        }

        searchBar.searchTextField.isEnabled = false
    }

    // MARK: - Private

    private func setUpDataSource() {
        dataSource = LocationDataSource(
            tableView: tableView,
            allLocations: AllLocationDataSource(),
            customLists: CustomListsDataSource(repository: customListRepository)
        )

        dataSource?.didSelectRelayLocations = { [weak self] relays in
            Task { @MainActor in
                self?.delegate?.didSelectRelays(relays: relays)
            }
        }

        dataSource?.didTapEditCustomLists = { [weak self] in
            guard let self else { return }

            Task { @MainActor in
                if let relaysWithLocation {
                    let allLocationDataSource = AllLocationDataSource()
                    allLocationDataSource.reload(relaysWithLocation)
                    delegate?.navigateToCustomLists(nodes: allLocationDataSource.nodes)
                }
            }
        }

        if let relaysWithLocation {
            dataSource?.setRelays(relaysWithLocation, selectedRelays: selectedRelays)
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
        tableView.setAccessibilityIdentifier(.selectLocationTableView)
    }

    private func setUpTopContent() {
        topContentView.axis = .vertical
        topContentView.addArrangedSubview(filterView)
        topContentView.addArrangedSubview(searchBar)
        filterView.setDaita(shouldFilterDaita)

        filterView.didUpdateFilter = { [weak self] in
            self?.delegate?.didUpdateFilter(filter: $0)
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
        searchBar.searchTextField.setAccessibilityIdentifier(.selectLocationSearchTextField)

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
