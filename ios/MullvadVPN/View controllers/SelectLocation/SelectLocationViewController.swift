//
//  SelectLocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadTypes
import RelayCache
import UIKit

final class SelectLocationViewController: UIViewController {
    private let searchBar = UISearchBar()
    private let tableView = UITableView()
    private var dataSource: LocationDataSource?
    private var cachedRelays: CachedRelays?
    var relayLocation: RelayLocation?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var didSelectRelay: ((RelayLocation) -> Void)?
    var didFinish: (() -> Void)?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "SelectLocation",
            value: "Select location",
            comment: ""
        )
        navigationItem.rightBarButtonItem = UIBarButtonItem(systemItem: .done, actionHandler: { [weak self] in
            self?.didFinish?()
        })

        setupDataSource()
        setupTableView()
        setupSearchBar()

        view.addConstrainedSubviews([searchBar, tableView]) {
            searchBar.pinEdgesToSuperviewMargins(.all().excluding(.bottom))

            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: searchBar.bottomAnchor)
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        tableView.flashScrollIndicators()
    }

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate(alongsideTransition: nil) { context in
            guard let indexPath = self.dataSource?.indexPathForSelectedRelay() else { return }

            self.tableView.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    // MARK: - Public

    func setCachedRelays(_ cachedRelays: CachedRelays) {
        self.cachedRelays = cachedRelays

        dataSource?.setRelays(cachedRelays.relays)
    }

    // MARK: - Private

    private func setupDataSource() {
        dataSource = LocationDataSource(tableView: tableView)
        dataSource?.didSelectRelayLocation = { [weak self] location in
            self?.didSelectRelay?(location)
        }

        dataSource?.selectedRelayLocation = relayLocation

        if let cachedRelays {
            dataSource?.setRelays(cachedRelays.relays)
        }
    }

    private func setupTableView() {
        tableView.backgroundColor = view.backgroundColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.estimatedRowHeight = 53
        tableView.indicatorStyle = .white
        tableView.keyboardDismissMode = .onDrag
    }

    private func setupSearchBar() {
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

        UISearchBar.SearchBarAppearance.inactive.apply(to: searchBar)
    }
}

extension SelectLocationViewController: UISearchBarDelegate {
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        dataSource?.filterRelays(by: searchText)
    }

    func searchBarTextDidBeginEditing(_ searchBar: UISearchBar) {
        UISearchBar.SearchBarAppearance.active.apply(to: searchBar)
    }

    func searchBarTextDidEndEditing(_ searchBar: UISearchBar) {
        UISearchBar.SearchBarAppearance.inactive.apply(to: searchBar)
    }
}
