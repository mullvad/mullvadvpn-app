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

protocol SelectLocationViewControllerDelegate: AnyObject {
    func selectLocationViewController(
        _ controller: SelectLocationViewController,
        didSelectRelayLocation relayLocation: RelayLocation
    )
}

class SelectLocationViewController: UIViewController, UITableViewDelegate {
    static let cellReuseIdentifier = "Cell"

    private var tableView: UITableView?

    private let tableHeaderFooterView = SelectLocationHeaderView()
    private var tableHeaderFooterViewTopConstraints: [NSLayoutConstraint] = []
    private var tableHeaderFooterViewBottomConstraints: [NSLayoutConstraint] = []

    private var dataSource: LocationDataSource?
    private var setCachedRelaysOnViewDidLoad: CachedRelays?
    private var setRelayLocationOnViewDidLoad: RelayLocation?
    private var setScrollPositionOnViewDidLoad: UITableView.ScrollPosition = .none
    private var isViewAppeared = false

    private var showHeaderViewAtTheBottom = false {
        didSet {
            setTableHeaderFooterConstraints()
        }
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    weak var delegate: SelectLocationViewControllerDelegate?
    var scrollToSelectedRelayOnViewWillAppear = true

    init() {
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationItem.largeTitleDisplayMode = .never
        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "SelectLocation",
            value: "Select Location",
            comment: ""
        )

        let tableView = UITableView(frame: view.bounds, style: .plain)
        tableView.translatesAutoresizingMaskIntoConstraints = false
        tableView.backgroundColor = .clear
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.estimatedRowHeight = 53
        tableView.indicatorStyle = .white

        tableView.register(
            SelectLocationCell.self,
            forCellReuseIdentifier: Self.cellReuseIdentifier
        )

        self.tableView = tableView

        view.accessibilityElements = [tableHeaderFooterView, tableView]
        view.backgroundColor = .secondaryColor
        view.addSubview(tableView)

        tableHeaderFooterView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(tableHeaderFooterView)

        dataSource = LocationDataSource(
            tableView: tableView,
            cellProvider: { tableView, indexPath, item in
                return tableView.dequeueReusableCell(
                    withIdentifier: Self.cellReuseIdentifier,
                    for: indexPath
                )
            },
            cellConfigurator: { [weak self] cell, indexPath, item in
                guard let cell = cell as? SelectLocationCell else { return }

                cell.accessibilityIdentifier = item.location.stringRepresentation
                cell.isDisabled = !item.isActive
                cell.locationLabel.text = item.displayName
                cell.showsCollapseControl = item.isCollapsible
                cell.isExpanded = item.showsChildren
                cell.didCollapseHandler = { [weak self] cell in
                    self?.collapseCell(cell)
                }
            }
        )

        tableView.delegate = self
        tableView.dataSource = dataSource

        tableHeaderFooterViewTopConstraints = [
            tableHeaderFooterView.topAnchor.constraint(equalTo: view.topAnchor),
            tableView.topAnchor.constraint(equalTo: tableHeaderFooterView.bottomAnchor),
            tableView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ]
        tableHeaderFooterViewBottomConstraints = [
            tableHeaderFooterView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            tableView.topAnchor.constraint(equalTo: view.topAnchor),
            tableView.bottomAnchor.constraint(equalTo: tableHeaderFooterView.topAnchor),
        ]

        NSLayoutConstraint.activate([
            tableHeaderFooterView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            tableHeaderFooterView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            tableView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            tableView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
        ])
        setTableHeaderFooterConstraints()

        if let setCachedRelaysOnViewDidLoad = setCachedRelaysOnViewDidLoad {
            dataSource?.setRelays(setCachedRelaysOnViewDidLoad.relays)
        }

        if let setRelayLocationOnViewDidLoad = setRelayLocationOnViewDidLoad {
            dataSource?.setSelectedRelayLocation(
                setRelayLocationOnViewDidLoad,
                showHiddenParents: true,
                animated: false,
                scrollPosition: setScrollPositionOnViewDidLoad
            )
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        // Show header view at the bottom when controller is presented inline and show header view
        // at the top of the view when controller is presented modally.
        showHeaderViewAtTheBottom = presentingViewController == nil

        if let indexPath = dataSource?.indexPathForSelectedRelay(),
           scrollToSelectedRelayOnViewWillAppear, !isViewAppeared
        {
            tableView?.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        isViewAppeared = true

        tableView?.flashScrollIndicators()
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)

        isViewAppeared = false
    }

    override func viewWillTransition(
        to size: CGSize,
        with coordinator: UIViewControllerTransitionCoordinator
    ) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate(alongsideTransition: nil) { context in
            guard let indexPath = self.dataSource?.indexPathForSelectedRelay() else { return }

            self.tableView?.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        updateTableHeaderTopLayoutMargin()
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        return dataSource?.item(for: indexPath)?.isActive ?? false
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        return dataSource?.item(for: indexPath)?.indentationLevel ?? 0
    }

    func tableView(
        _ tableView: UITableView,
        willDisplay cell: UITableViewCell,
        forRowAt indexPath: IndexPath
    ) {
        if let item = dataSource?.item(for: indexPath),
           item.location == dataSource?.selectedRelayLocation
        {
            cell.setSelected(true, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = dataSource?.item(for: indexPath) else { return }

        dataSource?.setSelectedRelayLocation(
            item.location,
            showHiddenParents: false,
            animated: false,
            scrollPosition: .none
        )

        delegate?.selectLocationViewController(self, didSelectRelayLocation: item.location)
    }

    // MARK: - Public

    func setCachedRelays(_ cachedRelays: CachedRelays) {
        guard isViewLoaded else {
            setCachedRelaysOnViewDidLoad = cachedRelays
            return
        }
        dataSource?.setRelays(cachedRelays.relays)
    }

    func setSelectedRelayLocation(
        _ relayLocation: RelayLocation?,
        animated: Bool,
        scrollPosition: UITableView.ScrollPosition
    ) {
        guard isViewLoaded else {
            setRelayLocationOnViewDidLoad = relayLocation
            setScrollPositionOnViewDidLoad = scrollPosition
            return
        }

        dataSource?.setSelectedRelayLocation(
            relayLocation,
            showHiddenParents: true,
            animated: animated,
            scrollPosition: scrollPosition
        )
    }

    // MARK: - Collapsible cells

    private func collapseCell(_ cell: SelectLocationCell) {
        guard let cellIndexPath = tableView?.indexPath(for: cell),
              let dataSource = dataSource,
              let location = dataSource.relayLocation(for: cellIndexPath)
        else {
            return
        }

        dataSource.toggleChildren(location, animated: true)
    }

    // MARK: - Private

    private func updateTableHeaderTopLayoutMargin() {
        // When contained within the navigation controller, we want the distance between
        // the navigation title and the table header label to be exactly 24pt.
        if let navigationBar = navigationController?.navigationBar as? CustomNavigationBar,
           !showHeaderViewAtTheBottom
        {
            tableHeaderFooterView.topLayoutMarginAdjustmentForNavigationBarTitle = navigationBar
                .titleLabelBottomInset
        } else {
            tableHeaderFooterView.topLayoutMarginAdjustmentForNavigationBarTitle = 0
        }
    }

    private func setTableHeaderFooterConstraints() {
        if showHeaderViewAtTheBottom {
            NSLayoutConstraint.deactivate(
                tableHeaderFooterViewTopConstraints
            )
            NSLayoutConstraint.activate(tableHeaderFooterViewBottomConstraints)
        } else {
            NSLayoutConstraint.deactivate(
                tableHeaderFooterViewBottomConstraints
            )
            NSLayoutConstraint.activate(tableHeaderFooterViewTopConstraints)
        }
        view.layoutIfNeeded()
    }
}
