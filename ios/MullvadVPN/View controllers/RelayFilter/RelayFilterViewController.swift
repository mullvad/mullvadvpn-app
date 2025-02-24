//
//  RelayFilterViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

class RelayFilterViewController: UIViewController {
    private let tableView = UITableView(frame: .zero, style: .grouped)
    private var viewModel: RelayFilterViewModel
    private var relayFilterManager: RelayFilterable
    private var dataSource: RelayFilterDataSource?
    private var disposeBag = Set<Combine.AnyCancellable>()

    private let explanationLabel: UILabel = {
        let label = UILabel()
        label.numberOfLines = 0
        label.lineBreakMode = .byWordWrapping
        label.font = .preferredFont(forTextStyle: .body)
        label.textColor = .secondaryLabel
        return label
    }()

    private let applyButton: AppButton = {
        let button = AppButton(style: .success)
        button.setAccessibilityIdentifier(.applyButton)
        button.setTitle(NSLocalizedString(
            "RELAY_FILTER_BUTTON_TITLE",
            tableName: "RelayFilter",
            value: "No matching servers",
            comment: ""
        ), for: .disabled)
        return button
    }()

    var onApplyFilter: ((RelayFilter) -> Void)?
    var didFinish: (() -> Void)?
    var onNewSettings: ((LatestTunnelSettings) -> Void)?
    var onNewRelays: ((LocationRelays) -> Void)?

    init(
        settings: LatestTunnelSettings,
        relays: LocationRelays,
        relayFilterManager: RelayFilterable
    ) {
        self.relayFilterManager = relayFilterManager
        self.viewModel = RelayFilterViewModel(
            settings: settings,
            relaysWithLocation: relays,
            relayFilterManager: relayFilterManager
        )
        super.init(nibName: nil, bundle: nil)
        self.onNewSettings = { [weak self] newSettings in
            guard let self else { return }
            viewModel.onNewSettings?(newSettings)
        }
        self.onNewRelays = { [weak self] newRelays in
            guard let self else { return }
            viewModel.onNewRelays?(newRelays)
            dataSource?.updateDataSnapshot()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

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

        view.addConstrainedSubviews([tableView, explanationLabel, applyButton]) {
            tableView.pinEdgesToSuperview(.all().excluding(.bottom))
            explanationLabel.pinEdgesToSuperviewMargins(PinnableEdges([.leading(.zero), .trailing(.zero)]))
            applyButton.pinEdgesToSuperviewMargins(.all().excluding(.top))
            applyButton.topAnchor.constraint(
                equalTo: explanationLabel.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            )

            explanationLabel.topAnchor.constraint(
                equalTo: tableView.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            )
        }

        setupDataSource()
    }

    private func setupDataSource() {
        viewModel.$relayFilter
            .sink { [weak self] filter in
                guard let self else { return }
                let filterDescriptor = viewModel.getFilteredRelays(filter)
                applyButton.isEnabled = filterDescriptor.isEnabled
                applyButton.setTitle(filterDescriptor.title, for: .normal)
                explanationLabel.text = filterDescriptor.description
            }
            .store(in: &disposeBag)
        dataSource = RelayFilterDataSource(tableView: tableView, viewModel: viewModel)
    }

    @objc private func applyFilter() {
        onApplyFilter?(viewModel.relayFilter)
    }
}
