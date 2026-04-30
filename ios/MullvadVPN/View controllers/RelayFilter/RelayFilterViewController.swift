//
//  RelayFilterViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

class RelayFilterViewController: UIViewController {
    private let tableView = UITableView(frame: .zero, style: .grouped)
    private var viewModel: RelayFilterViewModel
    private var dataSource: RelayFilterDataSource?
    private var disposeBag = Set<Combine.AnyCancellable>()

    private let buttonContainerView: UIStackView = {
        let containerView = UIStackView()
        containerView.axis = .vertical
        containerView.spacing = 16
        containerView.isLayoutMarginsRelativeArrangement = true
        return containerView
    }()

    private let descriptionContainer: UIStackView = {
        let container = UIStackView()
        container.axis = .vertical
        container.spacing = 16
        return container
    }()

    private let applyButton: AppButton = {
        let button = AppButton(style: .success)
        button.setAccessibilityIdentifier(.applyButton)
        return button
    }()

    var onApplyFilter: ((RelayFilter, MultihopContext) -> Void)?
    var didFinish: (() -> Void)?

    init(
        settings: LatestTunnelSettings,
        relaySelectorWrapper: RelaySelectorWrapper,
        multihopContext: MultihopContext
    ) {
        self.viewModel = RelayFilterViewModel(
            settings: settings,
            relaySelectorWrapper: relaySelectorWrapper,
            multihopContext: multihopContext
        )
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        view.backgroundColor = .secondaryColor

        if viewModel.multihopContext == .entry {
            navigationItem.title = NSLocalizedString("Entry filter", comment: "")
        } else {
            navigationItem.title = NSLocalizedString("Exit filter", comment: "")
        }

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
        tableView.isMultipleTouchEnabled = false

        view.addSubview(tableView)
        buttonContainerView.addArrangedSubview(descriptionContainer)
        buttonContainerView.addArrangedSubview(applyButton)

        view.addConstrainedSubviews([tableView, buttonContainerView]) {
            tableView.pinEdgesToSuperview(.all().excluding(.bottom))
            buttonContainerView.pinEdgesToSuperviewMargins(.all().excluding(.top))
            buttonContainerView.topAnchor.constraint(
                equalTo: tableView.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            )
        }

        setupDataSource()
    }

    private func setupDataSource() {
        viewModel
            .$relayFilter
            .removeDuplicates()
            .sink { [weak self] filter in
                guard let self else { return }
                let filterDescriptor = viewModel.getFilteredRelays(filter)
                applyButton.isEnabled = filterDescriptor.isEnabled
                applyButton.setTitle(filterDescriptor.title, for: .normal)

                descriptionContainer.subviews.forEach { $0.removeFromSuperview() }
                if filterDescriptor.descriptions.isEmpty {
                    descriptionContainer.isHidden = true
                } else {
                    descriptionContainer.isHidden = false

                    filterDescriptor.descriptions.forEach { description in
                        let label = UILabel()
                        label.numberOfLines = 0
                        label.lineBreakMode = .byWordWrapping
                        label.font = .mullvadTiny
                        label.adjustsFontForContentSizeCategory = true
                        label.textColor = .secondaryTextColor
                        label.text = description
                        label.textAlignment = .center

                        descriptionContainer.addArrangedSubview(label)
                    }
                }
            }
            .store(in: &disposeBag)
        dataSource = RelayFilterDataSource(tableView: tableView, viewModel: viewModel)
    }

    @objc private func applyFilter() {
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

        onApplyFilter?(relayFilter, viewModel.multihopContext)
    }
}
