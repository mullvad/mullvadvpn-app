//
//  SelectLocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

protocol SelectLocationViewControllerDelegate: AnyObject {
    func selectLocationViewController(_ controller: SelectLocationViewController, didSelectRelayLocation relayLocation: RelayLocation)
}

class SelectLocationViewController: UIViewController, UITableViewDelegate {

    private enum ReuseIdentifiers: String {
        case cell
        case header
    }

    private var tableView: UITableView?

    private let logger = Logger(label: "SelectLocationController")
    private var dataSource: LocationDataSource?
    private var setCachedRelaysOnViewDidLoad: CachedRelays?
    private var setRelayLocationOnViewDidLoad: RelayLocation?
    private var setScrollPositionOnViewDidLoad: UITableView.ScrollPosition = .none
    private var isViewAppeared = false

    private var showHeaderViewAtTheBottom = false {
        didSet {
            setTableHeaderFooterDimensions()
        }
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

        let tableView = UITableView(frame: view.bounds, style: .plain)
        tableView.translatesAutoresizingMaskIntoConstraints = true
        tableView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        tableView.backgroundColor = .clear
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.estimatedRowHeight = 53
        tableView.indicatorStyle = .white

        tableView.register(SelectLocationHeaderView.self, forHeaderFooterViewReuseIdentifier: ReuseIdentifiers.header.rawValue)
        tableView.register(SelectLocationCell.self, forCellReuseIdentifier: ReuseIdentifiers.cell.rawValue)

        self.tableView = tableView

        setTableHeaderFooterDimensions()

        view.backgroundColor = .secondaryColor
        view.addSubview(tableView)

        dataSource = LocationDataSource(
            tableView: tableView,
            cellProvider: { [weak self] (tableView, indexPath, item) -> UITableViewCell? in
                guard let self = self else { return nil }

                let cell = tableView.dequeueReusableCell(
                    withIdentifier: ReuseIdentifiers.cell.rawValue, for: indexPath) as! SelectLocationCell

                cell.accessibilityIdentifier = item.location.stringRepresentation
                cell.isDisabled = !item.isActive
                cell.locationLabel.text = item.displayName
                cell.statusIndicator.isActive = item.isActive
                cell.showsCollapseControl = item.isCollapsible
                cell.isExpanded = item.showsChildren
                cell.didCollapseHandler = { [weak self] (cell) in
                    self?.collapseCell(cell)
                }

                return cell
            })

        tableView.delegate = self
        tableView.dataSource = dataSource

        if let setCachedRelaysOnViewDidLoad = self.setCachedRelaysOnViewDidLoad {
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
        showHeaderViewAtTheBottom = self.presentingViewController == nil

        if let indexPath = dataSource?.indexPathForSelectedRelay(), scrollToSelectedRelayOnViewWillAppear, !isViewAppeared {
            self.tableView?.scrollToRow(at: indexPath, at: .middle, animated: false)
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

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate { (context) in
            if let indexPath = self.dataSource?.indexPathForSelectedRelay() {
                self.tableView?.scrollToRow(at: indexPath, at: .middle, animated: false)
            }
        }
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        return dataSource?.item(for: indexPath)?.isActive ?? false
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        return dataSource?.item(for: indexPath)?.indentationLevel ?? 0
    }

    func tableView(_ tableView: UITableView, willDisplay cell: UITableViewCell, forRowAt indexPath: IndexPath) {
        if let item = dataSource?.item(for: indexPath), item.location == dataSource?.selectedRelayLocation {
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

        self.delegate?.selectLocationViewController(self, didSelectRelayLocation: item.location)
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        assert(section == 0)

        if showHeaderViewAtTheBottom {
            return nil
        } else {
            let view = tableView.dequeueReusableHeaderFooterView(withIdentifier: ReuseIdentifiers.header.rawValue) as! SelectLocationHeaderView
            updateTopLayoutMargin(forHeaderView: view)

            return view
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        assert(section == 0)

        if showHeaderViewAtTheBottom {
            let view = tableView.dequeueReusableHeaderFooterView(withIdentifier: ReuseIdentifiers.header.rawValue) as! SelectLocationHeaderView
            view.topLayoutMarginAdjustmentForNavigationBarTitle = 0

            return view
        } else {
            return nil
        }
    }

    // MARK: - Public

    func setCachedRelays(_ cachedRelays: CachedRelays) {
        guard isViewLoaded else {
            self.setCachedRelaysOnViewDidLoad = cachedRelays
            return
        }
        self.dataSource?.setRelays(cachedRelays.relays)
    }

    func setSelectedRelayLocation(_ relayLocation: RelayLocation?, animated: Bool, scrollPosition: UITableView.ScrollPosition) {
        guard isViewLoaded else {
            self.setRelayLocationOnViewDidLoad = relayLocation
            self.setScrollPositionOnViewDidLoad = scrollPosition
            return
        }

        self.dataSource?.setSelectedRelayLocation(
            relayLocation,
            showHiddenParents: true,
            animated: animated,
            scrollPosition: scrollPosition
        )
    }

    // MARK: - Collapsible cells

    private func collapseCell(_ cell: SelectLocationCell) {
        guard let cellIndexPath = tableView?.indexPath(for: cell),
              let dataSource = dataSource, let location = dataSource.relayLocation(for: cellIndexPath) else {
            return
        }

        dataSource.toggleChildren(location, animated: true)
    }

    // MARK: - Private

    private func updateTopLayoutMargin(forHeaderView view: SelectLocationHeaderView) {
        // When contained within the navigation controller, we want the distance between the navigation title
        // and the table header label to be exactly 24pt.
        if let navigationBar = navigationController?.navigationBar as? CustomNavigationBar {
            view.topLayoutMarginAdjustmentForNavigationBarTitle = navigationBar.titleLabelBottomInset
        } else {
            view.topLayoutMarginAdjustmentForNavigationBarTitle = 0
        }
    }

    private func setTableHeaderFooterDimensions() {
        let headerFooterHeight: CGFloat = 109

        if showHeaderViewAtTheBottom {
            self.tableView?.estimatedSectionHeaderHeight = 0
            self.tableView?.estimatedSectionFooterHeight = headerFooterHeight
        } else {
            self.tableView?.estimatedSectionHeaderHeight = headerFooterHeight
            self.tableView?.estimatedSectionFooterHeight = 0
        }
    }
}
