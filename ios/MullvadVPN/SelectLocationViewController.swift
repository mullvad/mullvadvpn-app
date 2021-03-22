//
//  SelectLocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

class SelectLocationViewController: UIViewController, RelayCacheObserver, UITableViewDelegate {

    private enum ReuseIdentifiers: String {
        case cell
        case header
    }

    private lazy var tableView: UITableView = {
        let tableView = UITableView(frame: view.bounds, style: .plain)
        tableView.translatesAutoresizingMaskIntoConstraints = true
        tableView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        tableView.backgroundColor = .clear
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.estimatedRowHeight = 53
        tableView.estimatedSectionHeaderHeight = 109
        tableView.indicatorStyle = .white

        tableView.register(SelectLocationHeaderView.self, forHeaderFooterViewReuseIdentifier: ReuseIdentifiers.header.rawValue)
        tableView.register(SelectLocationCell.self, forCellReuseIdentifier: ReuseIdentifiers.cell.rawValue)

        return tableView
    }()

    private let logger = Logger(label: "SelectLocationController")
    private var dataSource: LocationDataSource?
    private var setCachedRelaysOnViewDidLoad: CachedRelays?
    private var setRelayLocationOnViewDidLoad: RelayLocation?
    private var isViewAppeared = false

    var didSelectRelayLocation: ((SelectLocationViewController, RelayLocation) -> Void)?
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

        view.backgroundColor = .secondaryColor
        view.addSubview(tableView)

        dataSource = LocationDataSource(
            tableView: self.tableView,
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
                scrollPosition: .none
            )
        }

        RelayCache.shared.addObserver(self)
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        if let indexPath = dataSource?.indexPathForSelectedRelay(), scrollToSelectedRelayOnViewWillAppear, !isViewAppeared {
            self.tableView.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        isViewAppeared = true

        tableView.flashScrollIndicators()
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)

        isViewAppeared = false
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
        didSelectRelayLocation?(self, item.location)
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        assert(section == 0)

        let view = tableView.dequeueReusableHeaderFooterView(withIdentifier: ReuseIdentifiers.header.rawValue) as! SelectLocationHeaderView

        // When contained within the navigation controller, we want the distance between the navigation title
        // and the table header label to be exactly 24pt.
        if let navigationBar = navigationController?.navigationBar as? CustomNavigationBar {
            view.topLayoutMarginAdjustmentForNavigationBarTitle = navigationBar.titleLabelBottomInset
        }

        return view
    }

    // MARK: - RelayCacheObserver

    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelays cachedRelays: CachedRelays) {
        DispatchQueue.main.async {
            self.didReceiveCachedRelays(cachedRelays)
        }
    }

    // MARK: - Public

    func prefetchData(completionHandler: @escaping (RelayCacheError?) -> Void) {
        RelayCache.shared.read { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let cachedRelays):
                    self.didReceiveCachedRelays(cachedRelays)
                    completionHandler(nil)

                case .failure(let error):
                    completionHandler(error)
                }
            }
        }
    }

    func setSelectedRelayLocation(_ relayLocation: RelayLocation?, animated: Bool, scrollPosition: UITableView.ScrollPosition) {
        guard isViewLoaded else {
            self.setRelayLocationOnViewDidLoad = relayLocation
            return
        }

        self.dataSource?.setSelectedRelayLocation(
            relayLocation,
            showHiddenParents: true,
            animated: animated,
            scrollPosition: scrollPosition
        )
    }

    // MARK: - Relay list handling

    private func didReceiveCachedRelays(_ cachedRelays: CachedRelays) {
        guard isViewLoaded else {
            self.setCachedRelaysOnViewDidLoad = cachedRelays
            return
        }
        self.dataSource?.setRelays(cachedRelays.relays)
    }

    // MARK: - Collapsible cells

    private func collapseCell(_ cell: SelectLocationCell) {
        guard let cellIndexPath = tableView.indexPath(for: cell),
              let dataSource = dataSource, let location = dataSource.relayLocation(for: cellIndexPath) else {
            return
        }

        dataSource.toggleChildren(location, animated: true)
    }
}
