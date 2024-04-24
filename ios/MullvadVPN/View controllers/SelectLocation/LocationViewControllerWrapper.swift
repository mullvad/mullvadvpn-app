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
    func didSelectRelays(relays: (relays: UserSelectedRelays, context: RelaySelection.MultihopContext))
    func didUpdateFilter(filter: RelayFilter)
}

final class LocationViewControllerWrapper: UIViewController {
    enum SegmentedControlOption: Int {
        case entry, exit
    }

    private let entryLocationViewController: LocationViewController?
    private let exitLocationViewController: LocationViewController
    private let segmentedControl = UISegmentedControl()
    private let locationViewContainer = UIStackView()
    private var selectedRelays: RelaySelection

    weak var delegate: LocationViewControllerWrapperDelegate?

    init(customListRepository: CustomListRepositoryProtocol, selectedRelays: RelaySelection) {
        entryLocationViewController = LocationViewController(
            customListRepository: customListRepository,
            selectedRelays: selectedRelays
        )

        exitLocationViewController = LocationViewController(
            customListRepository: customListRepository,
            selectedRelays: selectedRelays
        )

        self.selectedRelays = selectedRelays

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

    func setCachedRelays(_ cachedRelays: CachedRelays, filter: RelayFilter) {
        updateViewControllers {
            $0.setCachedRelays(cachedRelays, filter: filter)
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

        segmentedControl.insertSegment(withTitle: NSLocalizedString(
            "MULTIHOP_TAB_ENTRY",
            tableName: "SelectLocation",
            value: "Entry",
            comment: ""
        ), at: SegmentedControlOption.entry.rawValue, animated: false)
        segmentedControl.insertSegment(withTitle: NSLocalizedString(
            "MULTIHOP_TAB_EXIT",
            tableName: "SelectLocation",
            value: "Exit",
            comment: ""
        ), at: SegmentedControlOption.exit.rawValue, animated: false)

        segmentedControl.selectedSegmentIndex = selectedRelays.currentContext.rawValue
        segmentedControl.addTarget(self, action: #selector(segmentedControlDidChange), for: .valueChanged)
    }

    private func addSubviews() {
        view.addConstrainedSubviews([segmentedControl, locationViewContainer]) {
            segmentedControl.heightAnchor.constraint(equalToConstant: 44)
            segmentedControl.pinEdgesToSuperviewMargins(PinnableEdges([.top(0), .leading(8), .trailing(8)]))

            locationViewContainer.pinEdgesToSuperview(.all().excluding(.top))

            #if DEBUG
            locationViewContainer.topAnchor.constraint(equalTo: segmentedControl.bottomAnchor, constant: 4)
            #else
            locationViewContainer.pinEdgeToSuperviewMargin(.top(0))
            #endif
        }
    }

    @objc
    private func segmentedControlDidChange(sender: UISegmentedControl) {
        switch segmentedControl.selectedSegmentIndex {
        case SegmentedControlOption.entry.rawValue:
            selectedRelays.currentContext = .entry
        case SegmentedControlOption.exit.rawValue:
            selectedRelays.currentContext = .exit
        default:
            break
        }

        swapViewController()
    }

    func swapViewController() {
        locationViewContainer.arrangedSubviews.forEach { view in
            view.removeFromSuperview()
        }

        var currentViewController: LocationViewController?

        switch selectedRelays.currentContext {
        case .entry:
            exitLocationViewController.removeFromParent()
            currentViewController = entryLocationViewController
        case .exit:
            entryLocationViewController?.removeFromParent()
            currentViewController = exitLocationViewController
        }

        guard let currentViewController else { return }

        currentViewController.setSelectedRelays(selectedRelays)
        addChild(currentViewController)
        currentViewController.didMove(toParent: self)

        locationViewContainer.addArrangedSubview(currentViewController.view)
    }
}

extension LocationViewControllerWrapper: LocationViewControllerDelegate {
    func navigateToCustomLists(nodes: [LocationNode]) {
        delegate?.navigateToCustomLists(nodes: nodes)
    }

    func didSelectRelays(relays: (relays: UserSelectedRelays, context: RelaySelection.MultihopContext)) {
        switch relays.context {
        case .entry:
            selectedRelays.entry = relays.relays
            delegate?.didSelectRelays(relays: relays)

            // Trigger change in segmented control, which in turn triggers view controller swap.
            segmentedControl.selectedSegmentIndex = SegmentedControlOption.exit.rawValue
            segmentedControl.sendActions(for: .valueChanged)
        case .exit:
            delegate?.didSelectRelays(relays: relays)
            didFinish?()
        }
    }

    func didUpdateFilter(filter: RelayFilter) {
        delegate?.didUpdateFilter(filter: filter)
    }
}
