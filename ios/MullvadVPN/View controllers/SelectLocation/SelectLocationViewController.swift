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
        navigationItem.rightBarButtonItem = UIBarButtonItem(
            barButtonSystemItem: .done,
            target: self,
            action: #selector(handleDone)
        )

        setupDataSource()
        setupTableView()
        setupSearchBar()

        let searchBarTopMargin: CGFloat = splitViewController == nil ? 0 : 16
        view.addConstrainedSubviews([searchBar.searchTextField, tableView]) {
            searchBar.searchTextField.pinEdgesToSuperviewMargins(.init([.top(searchBarTopMargin)]))
            searchBar.searchTextField.pinEdgesToSuperview(.init([.leading(16), .trailing(16)]))

            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: searchBar.searchTextField.bottomAnchor, constant: 16)
        }
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

    @objc private func handleDone() {
        didFinish?()
    }

    private func setupDataSource() {
        dataSource = LocationDataSource(tableView: tableView)
        dataSource?.didSelectRelayLocation = { [weak self] location in
            self?.didSelectRelay?(location)
        }

        dataSource?.selectedRelayLocation = relayLocation

        if let cachedRelays = cachedRelays {
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
        tableView.dataSource = dataSource
    }

    private func setupSearchBar() {
        searchBar.delegate = self
        searchBar.layer.cornerRadius = 8
        searchBar.clipsToBounds = true
        searchBar.placeholder = NSLocalizedString(
            "SEARCHBAR_PLACEHOLDER",
            tableName: "SelectLocation",
            value: "Search for...",
            comment: ""
        )

        setSearchBarInactiveTheme()
    }

    private func setSearchBarActiveTheme() {
        searchBar.searchTextField.leftView?.tintColor = .primaryColor
        searchBar.searchTextField.tintColor = .black
        searchBar.searchTextField.textColor = .black
        searchBar.searchTextField.backgroundColor = .white
        searchBar.searchTextField.attributedPlaceholder = NSAttributedString(
            string: searchBar.placeholder ?? "",
            attributes: [
                .foregroundColor: UIColor.primaryColor,
            ]
        )
    }

    private func setSearchBarInactiveTheme() {
        searchBar.searchTextField.leftView?.tintColor = .white
        searchBar.searchTextField.tintColor = .white
        searchBar.searchTextField.textColor = .white
        searchBar.searchTextField.backgroundColor = .secondaryColor
        searchBar.searchTextField.attributedPlaceholder = NSAttributedString(
            string: searchBar.placeholder ?? "",
            attributes: [
                .foregroundColor: UIColor.white,
            ]
        )
    }
}

extension SelectLocationViewController: UISearchBarDelegate {
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        dataSource?.filterRelays(by: searchText)
    }

    func searchBarTextDidBeginEditing(_ searchBar: UISearchBar) {
        setSearchBarActiveTheme()
    }

    func searchBarTextDidEndEditing(_ searchBar: UISearchBar) {
        setSearchBarInactiveTheme()
    }
}
