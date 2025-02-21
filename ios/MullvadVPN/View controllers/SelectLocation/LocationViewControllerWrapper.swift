//
//  LocationViewControllerWrapper.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-04-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

protocol LocationViewControllerWrapperDelegate: AnyObject {
    func navigateToCustomLists(nodes: [LocationNode])
    func navigateToFilter()
    func navigateToDaitaSettings()
    func didSelectEntryRelays(_ relays: UserSelectedRelays)
    func didSelectExitRelays(_ relays: UserSelectedRelays)
    func didUpdateFilter(_ filter: RelayFilter)
}

final class LocationViewControllerWrapper: UIViewController {
    enum MultihopContext: Int, CaseIterable, CustomStringConvertible {
        case entry, exit

        var description: String {
            switch self {
            case .entry:
                NSLocalizedString(
                    "MULTIHOP_ENTRY",
                    tableName: "SelectLocation",
                    value: "Entry",
                    comment: ""
                )
            case .exit:
                NSLocalizedString(
                    "MULTIHOP_EXIT",
                    tableName: "SelectLocation",
                    value: "Exit",
                    comment: ""
                )
            }
        }
    }

    private var entryLocationViewController: LocationViewController?
    private let exitLocationViewController: LocationViewController
    private let segmentedControl = UISegmentedControl()
    private let locationViewContainer = UIView()
    private let relayFilterManager = RelayFilterManager()
    private var settings: LatestTunnelSettings
    private var relaysWithLocation: LocationRelays

    private var multihopContext: MultihopContext = .exit
    private var selectedEntry: UserSelectedRelays?
    private var selectedExit: UserSelectedRelays?

    weak var delegate: LocationViewControllerWrapperDelegate?

    var onNewSettings: ((LatestTunnelSettings) -> Void)?
    var onNewRelays: ((LocationRelays) -> Void)?

    private var relayFilter: RelayFilter {
        if case let .only(filter) = settings.relayConstraints.filter {
            return filter
        }
        return RelayFilter()
    }

    init(
        settings: LatestTunnelSettings,
        relays: LocationRelays,
        customListRepository: CustomListRepositoryProtocol
    ) {
        self.selectedEntry = settings.relayConstraints.entryLocations.value
        self.selectedExit = settings.relayConstraints.exitLocations.value
        self.settings = settings
        self.relaysWithLocation = relays

        entryLocationViewController = LocationViewController(
            customListRepository: customListRepository,
            selectedRelays: RelaySelection(),
            shouldFilterDaita: settings.daita.isDirectOnly
        )

        exitLocationViewController = LocationViewController(
            customListRepository: customListRepository,
            selectedRelays: RelaySelection(),
            shouldFilterDaita: settings.daita.isDirectOnly && !settings.daita.isAutomaticRouting
        )

        super.init(nibName: nil, bundle: nil)

        self.onNewSettings = { [weak self] newSettings in
            self?.settings = newSettings
            self?.setRelaysWithLocation()
        }

        self.onNewRelays = { [weak self] newRelays in
            self?.relaysWithLocation = newRelays
            self?.setRelaysWithLocation()
        }
        setRelaysWithLocation()

        updateViewControllers {
            $0.delegate = self
        }
    }

    var didFinish: (() -> Void)?

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.setAccessibilityIdentifier(.selectLocationViewWrapper)
        view.backgroundColor = .secondaryColor

        setUpNavigation()
        setUpSegmentedControl()
        addSubviews()
        add(entryLocationViewController)
        add(exitLocationViewController)
        swapViewController()
    }

    private func setRelaysWithLocation() {
        if settings.tunnelMultihopState.isEnabled {
            entryLocationViewController?.setDaitaChip(settings.daita.isDirectOnly)
            entryLocationViewController?.toggleDaitaAutomaticRouting(isEnabled: settings.daita.isAutomaticRouting)
            entryLocationViewController?.setRelaysWithLocation(
                relayFilterManager.matches(relays: relaysWithLocation, criteria: RelayFilterCriteria(
                    isDirectOnly: settings.daita.isDirectOnly,
                    filterConstraints: relayFilter
                )),
                filter: relayFilter
            )
            exitLocationViewController.setRelaysWithLocation(
                relayFilterManager.matches(relays: relaysWithLocation, criteria: RelayFilterCriteria(
                    isDirectOnly: false,
                    filterConstraints: relayFilter
                )),
                filter: relayFilter
            )
        } else {
            exitLocationViewController.setRelaysWithLocation(
                relayFilterManager.matches(relays: relaysWithLocation, criteria: RelayFilterCriteria(
                    isDirectOnly: settings.daita.isDirectOnly,
                    filterConstraints: relayFilter
                )),
                filter: relayFilter
            )
        }
    }

    func refreshCustomLists() {
        updateViewControllers {
            $0.refreshCustomLists()
        }
    }

    private func updateViewControllers(callback: (LocationViewController) -> Void) {
        [entryLocationViewController, exitLocationViewController]
            .compactMap { $0 }
            .forEach { callback($0) }
    }

    private func setUpNavigation() {
        navigationItem.largeTitleDisplayMode = .never

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "SelectLocation",
            value: "Select location",
            comment: ""
        )

        navigationItem.leftBarButtonItem = UIBarButtonItem(
            title: NSLocalizedString(
                "NAVIGATION_FILTER",
                tableName: "SelectLocation",
                value: "Filter",
                comment: ""
            ),
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.delegate?.navigateToFilter()
            })
        )
        navigationItem.leftBarButtonItem?.setAccessibilityIdentifier(.selectLocationFilterButton)

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.didFinish?()
            })
        )
        navigationItem.rightBarButtonItem?.setAccessibilityIdentifier(.closeSelectLocationButton)
    }

    private func setUpSegmentedControl() {
        segmentedControl.backgroundColor = .SegmentedControl.backgroundColor
        segmentedControl.selectedSegmentTintColor = .SegmentedControl.selectedColor
        segmentedControl.setTitleTextAttributes([
            .foregroundColor: UIColor.white,
            .font: UIFont.systemFont(ofSize: 17, weight: .medium),
        ], for: .normal)

        segmentedControl.insertSegment(
            withTitle: MultihopContext.entry.description,
            at: MultihopContext.entry.rawValue,
            animated: false
        )
        segmentedControl.insertSegment(
            withTitle: MultihopContext.exit.description,
            at: MultihopContext.exit.rawValue,
            animated: false
        )

        segmentedControl.selectedSegmentIndex = multihopContext.rawValue
        segmentedControl.addTarget(self, action: #selector(segmentedControlDidChange), for: .valueChanged)
    }

    private func addSubviews() {
        view.addConstrainedSubviews([segmentedControl, locationViewContainer]) {
            segmentedControl.heightAnchor.constraint(equalToConstant: 44)
            segmentedControl.pinEdgesToSuperviewMargins(PinnableEdges([.top(0), .leading(8), .trailing(8)]))

            locationViewContainer.pinEdgesToSuperview(.all().excluding(.top))

            if settings.tunnelMultihopState.isEnabled {
                locationViewContainer.topAnchor.constraint(equalTo: segmentedControl.bottomAnchor, constant: 4)
            } else {
                locationViewContainer.pinEdgeToSuperviewMargin(.top(0))
            }
        }
    }

    private func add(_ locationViewController: LocationViewController?) {
        guard let locationViewController else { return }
        addChild(locationViewController)
        locationViewController.didMove(toParent: self)
        locationViewContainer.addConstrainedSubviews([locationViewController.view]) {
            locationViewController.view.pinEdgesToSuperview()
        }
    }

    @objc
    private func segmentedControlDidChange(sender: UISegmentedControl) {
        multihopContext = .allCases[segmentedControl.selectedSegmentIndex]
        swapViewController()
    }

    private func swapViewController() {
        var selectedRelays: RelaySelection
        var oldViewController: LocationViewController?
        var newViewController: LocationViewController?

        (selectedRelays, oldViewController, newViewController) = switch multihopContext {
        case .entry:
            (
                RelaySelection(
                    selected: selectedEntry,
                    excluded: selectedExit,
                    excludedTitle: MultihopContext.exit.description
                ),
                exitLocationViewController,
                entryLocationViewController
            )
        case .exit:
            (
                RelaySelection(
                    selected: selectedExit,
                    excluded: settings.tunnelMultihopState.isEnabled ? selectedEntry : nil,
                    excludedTitle: MultihopContext.entry.description
                ),
                entryLocationViewController,
                exitLocationViewController
            )
        }
        newViewController?.setSelectedRelays(selectedRelays)
        oldViewController?.view.isUserInteractionEnabled = false
        newViewController?.view.isUserInteractionEnabled = true
        UIView.animate(withDuration: 0.0) {
            oldViewController?.view.alpha = 0
            newViewController?.view.alpha = 1
        }
    }
}

extension LocationViewControllerWrapper: @preconcurrency LocationViewControllerDelegate {
    func navigateToCustomLists(nodes: [LocationNode]) {
        delegate?.navigateToCustomLists(nodes: nodes)
    }

    func navigateToDaitaSettings() {
        delegate?.navigateToDaitaSettings()
    }

    func didSelectRelays(relays: UserSelectedRelays) {
        switch multihopContext {
        case .entry:
            selectedEntry = relays
            delegate?.didSelectEntryRelays(relays)

            // Trigger change in segmented control, which in turn triggers view controller swap.
            segmentedControl.selectedSegmentIndex = MultihopContext.exit.rawValue
            segmentedControl.sendActions(for: .valueChanged)
        case .exit:
            delegate?.didSelectExitRelays(relays)
            didFinish?()
        }
    }

    func didUpdateFilter(filter: RelayFilter) {
        delegate?.didUpdateFilter(filter)
    }
}
