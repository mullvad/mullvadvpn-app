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

final class SelectLocationViewController: UIViewController, UITableViewDelegate {
    private var tableView: UITableView?
    private var dataSource: LocationDataSource?
    private var cachedRelays: CachedRelays?
    private var relayLocation: RelayLocation?
    private var scrollPosition: UITableView.ScrollPosition?
    private var isViewAppeared = false

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var didSelectRelay: ((RelayLocation) -> Void)?
    var didFinish: (() -> Void)?

    var scrollToSelectedRelayOnViewWillAppear = true

    init() {
        super.init(style: .plain)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

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

        setupTableView()
        setupDataSource()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        if let indexPath = dataSource?.indexPathForSelectedRelay(),
           scrollToSelectedRelayOnViewWillAppear, !isViewAppeared
        {
            tableView?.scrollToRow(at: indexPath, at: scrollPosition ?? .middle, animated: false)
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        isViewAppeared = true
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)

        isViewAppeared = false
    }

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate(alongsideTransition: nil) { context in
            guard let indexPath = self.dataSource?.indexPathForSelectedRelay() else { return }

            self.tableView?.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    // MARK: - Public

    func setCachedRelays(_ cachedRelays: CachedRelays) {
        self.cachedRelays = cachedRelays

        dataSource?.setRelays(cachedRelays.relays)
    }

    func setSelectedRelayLocation(
        _ relayLocation: RelayLocation?,
        animated: Bool,
        scrollPosition: UITableView.ScrollPosition
    ) {
        self.relayLocation = relayLocation
        self.scrollPosition = scrollPosition

        dataSource?.setSelectedRelayLocation(relayLocation, animated: animated)
    }

    // MARK: - Private

    private func setupTableView() {
        let tableView = UITableView(frame: view.bounds, style: .plain)
        tableView.backgroundColor = .clear
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.estimatedRowHeight = 53
        tableView.indicatorStyle = .white
        tableView.delegate = self

        view.backgroundColor = .secondaryColor

        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }

        self.tableView = tableView
    }

    private func setupDataSource() {
        guard let tableView = tableView else { return }

        dataSource = LocationDataSource(tableView: tableView)
        dataSource?.didSelectRelayLocation = { [weak self] location in
            self?.didSelectRelay?(location)
        }

        if let cachedRelays = cachedRelays {
            dataSource?.setRelays(cachedRelays.relays)
        }

        if let relayLocation = relayLocation {
            dataSource?.setSelectedRelayLocation(relayLocation, animated: false)
        }
    }

    @objc private func handleDone() {
        didFinish?()
    }
}
