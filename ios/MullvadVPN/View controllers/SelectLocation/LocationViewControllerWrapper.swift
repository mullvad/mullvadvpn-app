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
    func didSelectRelays(relays: UserSelectedRelays)
    func didUpdateFilter(filter: RelayFilter)
}

final class LocationViewControllerWrapper: UIViewController {
    private let locationViewController: LocationViewController
    private let segmentedControl = UISegmentedControl()

    weak var delegate: LocationViewControllerWrapperDelegate?

    init(customListRepository: CustomListRepositoryProtocol, selectedRelays: UserSelectedRelays?) {
        locationViewController = LocationViewController(
            customListRepository: customListRepository,
            selectedRelays: selectedRelays
        )

        super.init(nibName: nil, bundle: nil)

        locationViewController.delegate = self
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
    }

    func setCachedRelays(_ cachedRelays: CachedRelays, filter: RelayFilter) {
        locationViewController.setCachedRelays(cachedRelays, filter: filter)
    }

    func refreshCustomLists() {
        locationViewController.refreshCustomLists()
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
        ), at: 0, animated: false)
        segmentedControl.insertSegment(withTitle: NSLocalizedString(
            "MULTIHOP_TAB_EXIT",
            tableName: "SelectLocation",
            value: "Exit",
            comment: ""
        ), at: 1, animated: false)

        segmentedControl.selectedSegmentIndex = 0
        segmentedControl.addTarget(self, action: #selector(segmentedControlDidChange), for: .valueChanged)
    }

    private func addSubviews() {
        addChild(locationViewController)
        locationViewController.didMove(toParent: self)

        view.addConstrainedSubviews([segmentedControl, locationViewController.view]) {
            segmentedControl.heightAnchor.constraint(equalToConstant: 44)
            segmentedControl.pinEdgesToSuperviewMargins(PinnableEdges([.top(0), .leading(8), .trailing(8)]))

            locationViewController.view.pinEdgesToSuperview(.all().excluding(.top))

            #if DEBUG
            locationViewController.view.topAnchor.constraint(equalTo: segmentedControl.bottomAnchor, constant: 4)
            #else
            locationViewController.view.pinEdgeToSuperviewMargin(.top(0))
            #endif
        }
    }

    @objc
    private func segmentedControlDidChange(sender: UISegmentedControl) {
        refreshCustomLists()
    }
}

extension LocationViewControllerWrapper: LocationViewControllerDelegate {
    func navigateToCustomLists(nodes: [LocationNode]) {
        delegate?.navigateToCustomLists(nodes: nodes)
    }

    func didSelectRelays(relays: UserSelectedRelays) {
        delegate?.didSelectRelays(relays: relays)
    }

    func didUpdateFilter(filter: RelayFilter) {
        delegate?.didUpdateFilter(filter: filter)
    }
}
