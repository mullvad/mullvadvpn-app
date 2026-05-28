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
import SwiftUI
import UIKit

extension RelayFilterSelection {
    class ViewController: UIViewController {
        private let tableView = UITableView(frame: .zero, style: .grouped)
        private var viewModel: ViewModel
        private var dataSource: DataSource?
        private var disposeBag = Set<Combine.AnyCancellable>()

        private let filterSettingsView: UIView
        private let buttonContainerView = UIStackView(
            axis: .vertical, isLayoutMarginsRelativeArrangement: true, spacing: 16)
        private let descriptionContainer = UIStackView(axis: .vertical, spacing: 16)
        private var filterSettingsTopConstraint: NSLayoutConstraint!
        private var filterSettingsTableViewConstraint: NSLayoutConstraint!
        private var tableViewTopConstraint: NSLayoutConstraint!

        private let applyButton: AppButton = {
            let button = AppButton(style: .success)
            button.setAccessibilityIdentifier(.applyButton)
            return button
        }()

        var onApplyFilter: ((RelayFilter, MultihopContext) -> Void)?
        var didFinish: (() -> Void)?

        init(viewModel: ViewModel) {
            self.viewModel = viewModel
            filterSettingsView = UIHostingController(rootView: SettingsView(viewModel: viewModel)).view
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

            filterSettingsView.backgroundColor = .secondaryColor

            view.addConstrainedSubviews([filterSettingsView, tableView, buttonContainerView]) {
                filterSettingsView.pinEdgesToSuperview(.all().excluding([.top, .bottom]))
                tableView.pinEdgesToSuperview(.all().excluding([.top, .bottom]))
                buttonContainerView.pinEdgesToSuperviewMargins(.all().excluding(.top))
                buttonContainerView.topAnchor.constraint(
                    equalTo: tableView.bottomAnchor,
                    constant: UIMetrics.contentLayoutMargins.top
                )
            }
            filterSettingsTopConstraint = filterSettingsView.topAnchor.constraint(
                equalTo: view.safeAreaLayoutGuide.topAnchor)
            filterSettingsTableViewConstraint = tableView.topAnchor.constraint(
                equalTo: filterSettingsView.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            )
            tableViewTopConstraint = tableView.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor)
            adjustFilterSettingsVisibility(
                filtersActive: viewModel.multihopContext == .entry && !viewModel.filters.isEmpty)
            setupDataSource()
        }

        private func adjustFilterSettingsVisibility(filtersActive: Bool) {
            let filterSettingsIsVisible = viewModel.multihopContext == .entry && filtersActive
            filterSettingsTopConstraint.isActive = filterSettingsIsVisible
            filterSettingsTableViewConstraint.isActive = filterSettingsIsVisible
            tableViewTopConstraint.isActive = !filterSettingsIsVisible
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
            viewModel
                .$filters
                .sink { [weak self] filters in
                    self?.adjustFilterSettingsVisibility(filtersActive: !filters.isEmpty)
                }
                .store(in: &disposeBag)
            dataSource = DataSource(tableView: tableView, viewModel: viewModel)
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
}
