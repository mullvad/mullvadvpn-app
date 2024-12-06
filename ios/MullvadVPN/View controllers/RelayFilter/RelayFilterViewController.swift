//
//  RelayFilterViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadTypes
import UIKit

class RelayFilterViewController: UIViewController {
    private let tableView = UITableView(frame: .zero, style: .grouped)
    private var viewModel: RelayFilterViewModel?
    private var dataSource: RelayFilterDataSource?
    private var cachedRelays: CachedRelays?
    private var filter = RelayFilter()
    private var disposeBag = Set<Combine.AnyCancellable>()

    private let applyButton: AppButton = {
        let button = AppButton(style: .success)
        button.setAccessibilityIdentifier(.applyButton)
        button.setTitle(NSLocalizedString(
            "RELAY_FILTER_BUTTON_TITLE",
            tableName: "RelayFilter",
            value: "Apply",
            comment: ""
        ), for: .normal)
        return button
    }()

    var onApplyFilter: ((RelayFilter) -> Void)?
    var didFinish: (() -> Void)?

    override func viewDidLoad() {
        super.viewDidLoad()

        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        view.backgroundColor = .secondaryColor

        navigationItem.title = NSLocalizedString(
            "RELAY_FILTER_NAVIGATION_TITLE",
            tableName: "RelayFilter",
            value: "Filter",
            comment: ""
        )

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.didFinish?()
            })
        )

        applyButton.addTarget(self, action: #selector(applyFilter), for: .touchUpInside)

        tableView.backgroundColor = view.backgroundColor
        tableView.separatorColor = view.backgroundColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60
        tableView.estimatedSectionHeaderHeight = tableView.estimatedRowHeight
        tableView.allowsMultipleSelection = true

        view.addConstrainedSubviews([tableView, applyButton]) {
            tableView.pinEdgesToSuperview(.all().excluding(.bottom))
            applyButton.pinEdgesToSuperviewMargins(.all().excluding(.top))
            applyButton.topAnchor.constraint(
                equalTo: tableView.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            )
        }

        setUpDataSource()
    }

    func setCachedRelays(_ cachedRelays: CachedRelays, filter: RelayFilter) {
        self.cachedRelays = cachedRelays
        self.filter = filter

        viewModel?.relays = cachedRelays.relays.wireguard.relays
        viewModel?.relayFilter = filter
    }

    private func setUpDataSource() {
        let viewModel = RelayFilterViewModel(
            relays: cachedRelays?.relays.wireguard.relays ?? [],
            relayFilter: filter
        )
        self.viewModel = viewModel

        viewModel.$relayFilter
            .sink { [weak self] filter in
                switch filter.providers {
                case .any:
                    self?.applyButton.isEnabled = true
                case let .only(providers):
                    switch filter.ownership {
                    case .any:
                        self?.applyButton.isEnabled = !providers.isEmpty
                    case .owned:
                        let filterHasAtLeastOneOwnedProvider = viewModel.ownedProviders
                            .first(where: { providers.contains($0) }) != nil
                        self?.applyButton.isEnabled = filterHasAtLeastOneOwnedProvider
                    case .rented:
                        let filterHasAtLeastOneRentedProvider = viewModel.rentedProviders
                            .first(where: { providers.contains($0) }) != nil
                        self?.applyButton.isEnabled = filterHasAtLeastOneRentedProvider
                    }
                }
            }
            .store(in: &disposeBag)

        dataSource = RelayFilterDataSource(tableView: tableView, viewModel: viewModel)
    }

    @objc private func applyFilter() {
        guard let viewModel = viewModel else { return }
        var relayFilter = viewModel.relayFilter

        switch viewModel.relayFilter.ownership {
        case .any:
            break
        case .owned:
            switch relayFilter.providers {
            case .any:
                break
            case let .only(providers):
                let ownedProviders = viewModel.ownedProviders.filter { providers.contains($0) }
                relayFilter.providers = .only(ownedProviders)
            }
        case .rented:
            switch relayFilter.providers {
            case .any:
                break
            case let .only(providers):
                let rentedProviders = viewModel.rentedProviders.filter { providers.contains($0) }
                relayFilter.providers = .only(rentedProviders)
            }
        }

        onApplyFilter?(relayFilter)
    }
}
