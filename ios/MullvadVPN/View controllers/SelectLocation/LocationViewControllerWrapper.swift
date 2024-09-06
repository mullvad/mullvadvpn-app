//
//  LocationViewControllerWrapper.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-04-23.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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

    private var entryLocationViewController: LocationViewController
    private let exitLocationViewController: LocationViewController
    private let segmentedControl = UISegmentedControl()
    private let locationViewContainer = UIStackView()

    private var selectedEntry: UserSelectedRelays?
    private var selectedExit: UserSelectedRelays?
    private let multihopEnabled: Bool
    private var multihopContext: MultihopContext = .exit
    private var daitaSettings: DAITASettings

    weak var delegate: LocationViewControllerWrapperDelegate?

    init(
        customListRepository: CustomListRepositoryProtocol,
        constraints: RelayConstraints,
        multihopEnabled: Bool,
        daitaSettings: DAITASettings,
        startContext: MultihopContext
    ) {
        self.multihopEnabled = multihopEnabled
        self.daitaSettings = daitaSettings
        multihopContext = startContext

        selectedEntry = constraints.entryLocations.value
        selectedExit = constraints.exitLocations.value

        entryLocationViewController = LocationViewController(
            customListRepository: customListRepository,
            selectedRelays: RelaySelection(),
            shouldFilterDaita: daitaSettings.shouldDoDirectOnly
        )

        if daitaSettings.shouldDoAutomaticRouting {
            entryLocationViewController.addDaitaInfoView()
        }

        exitLocationViewController = LocationViewController(
            customListRepository: customListRepository,
            selectedRelays: RelaySelection(),
            shouldFilterDaita: daitaSettings.shouldDoDirectOnly && !multihopEnabled
        )

        super.init(nibName: nil, bundle: nil)

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

        view.accessibilityIdentifier = .selectLocationViewWrapper
        view.backgroundColor = .secondaryColor

        setUpNavigation()
        setUpSegmentedControl()
        addSubviews()
        swapViewController()
    }

    func setRelaysWithLocation(_ relaysWithLocation: LocationRelays, filter: RelayFilter) {
        var daitaFilteredRelays = relaysWithLocation
        if daitaSettings.daitaState.isEnabled && daitaSettings.directOnlyState.isEnabled {
            daitaFilteredRelays.relays = relaysWithLocation.relays.filter { relay in
                relay.daita == true
            }
        }

        if multihopEnabled {
            entryLocationViewController.setRelaysWithLocation(daitaFilteredRelays, filter: filter)
            exitLocationViewController.setRelaysWithLocation(relaysWithLocation, filter: filter)
        } else {
            exitLocationViewController.setRelaysWithLocation(daitaFilteredRelays, filter: filter)
        }
    }

    func refreshCustomLists() {
        updateViewControllers {
            $0.refreshCustomLists()
        }
    }

    func onDaitaSettingsUpdate(_ settings: DAITASettings, relaysWithLocation: LocationRelays, filter: RelayFilter) {
        daitaSettings = settings
        guard multihopEnabled else { return }

        setRelaysWithLocation(relaysWithLocation, filter: filter)
        entryLocationViewController.setShouldFilterDaita(settings.shouldDoDirectOnly)

        if daitaSettings.shouldDoAutomaticRouting {
            entryLocationViewController.addDaitaInfoView()
        } else if daitaSettings.shouldDoDirectOnly {
            entryLocationViewController.removeDaitaInfoView()
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
        navigationItem.leftBarButtonItem?.accessibilityIdentifier = .selectLocationFilterButton

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.didFinish?()
            })
        )
        navigationItem.rightBarButtonItem?.accessibilityIdentifier = .closeSelectLocationButton
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

            if multihopEnabled {
                locationViewContainer.topAnchor.constraint(equalTo: segmentedControl.bottomAnchor, constant: 4)
            } else {
                locationViewContainer.pinEdgeToSuperviewMargin(.top(0))
            }
        }
    }

    @objc
    private func segmentedControlDidChange(sender: UISegmentedControl) {
        multihopContext = .allCases[segmentedControl.selectedSegmentIndex]
        swapViewController()
    }

    func swapViewController() {
        locationViewContainer.arrangedSubviews.forEach { view in
            view.removeFromSuperview()
        }

        let (selectedRelays, oldViewController, newViewController) = switch multihopContext {
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
                    excluded: multihopEnabled ? selectedEntry : nil,
                    excludedTitle: MultihopContext.entry.description
                ),
                entryLocationViewController,
                exitLocationViewController
            )
        }

        oldViewController.removeFromParent()
        newViewController.setSelectedRelays(selectedRelays)
        addChild(newViewController)
        newViewController.didMove(toParent: self)

        locationViewContainer.addArrangedSubview(newViewController.view)
    }
}

extension LocationViewControllerWrapper: LocationViewControllerDelegate {
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
