//
//  ProxyConfigurationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 21/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

/// The view controller providing the interface for editing and testing the proxy configuration.
class ProxyConfigurationViewController: UIViewController, UITableViewDelegate {
    private let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    private let interactor: ProxyConfigurationInteractorProtocol
    private var cancellables = Set<AnyCancellable>()

    private var dataSource: UITableViewDiffableDataSource<
        ProxyConfigurationSectionIdentifier,
        ProxyConfigurationItemIdentifier
    >?
    private lazy var testBarButton: UIBarButtonItem = {
        let barButtonItem = UIBarButtonItem(
            title: NSLocalizedString("TEST_NAVIGATION_BUTTON", tableName: "APIAccess", value: "Test", comment: ""),
            primaryAction: UIAction { [weak self] _ in
                self?.onTest()
            }
        )
        barButtonItem.style = .done
        return barButtonItem
    }()

    private let contentController = UITableViewController(style: .insetGrouped)
    private var tableView: UITableView {
        contentController.tableView
    }

    private lazy var sheetPresentation: AccessMethodActionSheetPresentation = {
        let sheetPresentation = AccessMethodActionSheetPresentation()
        sheetPresentation.delegate = self
        return sheetPresentation
    }()

    weak var delegate: ProxyConfigurationViewControllerDelegate?

    init(subject: CurrentValueSubject<AccessMethodViewModel, Never>, interactor: ProxyConfigurationInteractorProtocol) {
        self.subject = subject
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        view.backgroundColor = .secondaryColor

        configureTableView()
        configureNavigationItem()
        configureDataSource()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        updateTableSafeAreaInsets()
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        guard let sectionIdentifier = dataSource?.snapshot().sectionIdentifiers[section] else { return nil }

        guard let headerView = tableView
            .dequeueReusableView(withIdentifier: AccessMethodHeaderFooterReuseIdentifier.primary)
        else { return nil }

        var contentConfiguration = UIListContentConfiguration.mullvadGroupedHeader()
        contentConfiguration.text = sectionIdentifier.sectionName

        headerView.contentConfiguration = contentConfiguration

        return headerView
    }

    func tableView(_ tableView: UITableView, willSelectRowAt indexPath: IndexPath) -> IndexPath? {
        guard let itemIdentifier = dataSource?.itemIdentifier(for: indexPath),
              itemIdentifier.isSelectable else { return nil }

        return indexPath
    }

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        guard let itemIdentifier = dataSource?.itemIdentifier(for: indexPath) else { return false }

        return itemIdentifier.isSelectable
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let itemIdentifier = dataSource?.itemIdentifier(for: indexPath)

        switch itemIdentifier {
        case .protocol:
            showProtocolSelector()
        case .proxyConfiguration(.shadowsocks(.cipher)):
            showShadowsocksCipher()
        default:
            break
        }
    }

    // MARK: - Pickers handling

    private func showProtocolSelector() {
        view.endEditing(false)
        delegate?.controllerShouldShowProtocolPicker(self)
    }

    private func showShadowsocksCipher() {
        view.endEditing(false)
        delegate?.controllerShouldShowShadowsocksCipherPicker(self)
    }

    // MARK: - Cell configuration

    private func dequeueCell(at indexPath: IndexPath, for itemIdentifier: ProxyConfigurationItemIdentifier)
        -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: itemIdentifier.cellIdentifier, for: indexPath)

        if let cell = cell as? DynamicBackgroundConfiguration {
            cell.setAutoAdaptingBackgroundConfiguration(.mullvadListGroupedCell(), selectionType: .dimmed)
        }

        switch itemIdentifier {
        case .protocol:
            configureProtocol(cell, itemIdentifier: itemIdentifier)
        case let .proxyConfiguration(proxyItemIdentifier):
            configureProxy(cell, itemIdentifier: proxyItemIdentifier)
        }

        return cell
    }

    private func configureProxy(_ cell: UITableViewCell, itemIdentifier: ProxyProtocolConfigurationItemIdentifier) {
        switch itemIdentifier {
        case let .socks(socksItemIdentifier):
            let section = SocksSectionHandler(tableStyle: tableView.style, subject: subject)
            section.configure(cell, itemIdentifier: socksItemIdentifier)

        case let .shadowsocks(shadowsocksItemIdentifier):
            let section = ShadowsocksSectionHandler(tableStyle: tableView.style, subject: subject)
            section.configure(cell, itemIdentifier: shadowsocksItemIdentifier)
        }
    }

    private func configureProtocol(_ cell: UITableViewCell, itemIdentifier: ProxyConfigurationItemIdentifier) {
        var contentConfiguration = UIListContentConfiguration.mullvadValueCell(tableStyle: tableView.style)
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.secondaryText = subject.value.method.localizedDescription
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }
    }

    // MARK: - Data source handling

    private func configureDataSource() {
        tableView.registerReusableViews(from: AccessMethodCellReuseIdentifier.self)
        tableView.registerReusableViews(from: AccessMethodHeaderFooterReuseIdentifier.self)

        dataSource = UITableViewDiffableDataSource(
            tableView: tableView,
            cellProvider: { [weak self] _, indexPath, itemIdentifier in
                self?.dequeueCell(at: indexPath, for: itemIdentifier)
            }
        )

        subject.withPreviousValue()
            .sink { [weak self] previousValue, newValue in
                self?.viewModelDidChange(previousValue: previousValue, newValue: newValue)
            }
            .store(in: &cancellables)
    }

    private func viewModelDidChange(previousValue: AccessMethodViewModel?, newValue: AccessMethodViewModel) {
        let animated = view.window != nil

        updateDataSource(previousValue: previousValue, newValue: newValue, animated: animated)
        updateSheet(previousValue: previousValue, newValue: newValue, animated: animated)
        validate()
    }

    private func updateSheet(previousValue: AccessMethodViewModel?, newValue: AccessMethodViewModel, animated: Bool) {
        guard previousValue?.testingStatus != newValue.testingStatus else { return }

        switch newValue.testingStatus {
        case .initial:
            sheetPresentation.hide(animated: animated)

        case .inProgress, .failed, .succeeded:
            var presentationConfiguration = AccessMethodActionSheetPresentationConfiguration()
            presentationConfiguration.dimsBackground = newValue.testingStatus == .inProgress
            presentationConfiguration.sheetConfiguration.context = .proxyConfiguration
            presentationConfiguration.sheetConfiguration.contentConfiguration.status = newValue.testingStatus
                .sheetStatus
            sheetPresentation.configuration = presentationConfiguration

            sheetPresentation.show(in: view, animated: animated)
        }
    }

    private func updateDataSource(
        previousValue: AccessMethodViewModel?,
        newValue: AccessMethodViewModel,
        animated: Bool
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<
            ProxyConfigurationSectionIdentifier,
            ProxyConfigurationItemIdentifier
        >()

        snapshot.appendSections([.protocol])
        snapshot.appendItems([.protocol], toSection: .protocol)
        // Reconfigure protocol cell on change.
        if let previousValue, previousValue.method != newValue.method {
            snapshot.reconfigureOrReloadItems([.protocol])
        }

        // Add proxy configuration section if the access method is configurable.
        if newValue.method.hasProxyConfiguration {
            snapshot.appendSections([.proxyConfiguration])
        }

        switch newValue.method {
        case .direct, .bridges:
            break

        case .shadowsocks:
            snapshot.appendItems(ProxyConfigurationItemIdentifier.allShadowsocksItems, toSection: .proxyConfiguration)
            // Reconfigure cipher cell on change.
            if let previousValue, previousValue.shadowsocks.cipher != newValue.shadowsocks.cipher {
                snapshot.reconfigureOrReloadItems([.proxyConfiguration(.shadowsocks(.cipher))])
            }

        case .socks5:
            snapshot.appendItems(
                ProxyConfigurationItemIdentifier.allSocksItems(authenticate: newValue.socks.authenticate),
                toSection: .proxyConfiguration
            )
        }

        dataSource?.apply(snapshot, animatingDifferences: animated)
    }

    private func validate() {
        let validationResult = Result { try subject.value.validate() }
        testBarButton.isEnabled = validationResult.isSuccess && subject.value.testingStatus != .inProgress
    }

    // MARK: - Misc

    private func configureTableView() {
        tableView.delegate = self
        tableView.backgroundColor = .secondaryColor

        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }

        addChild(contentController)
        contentController.didMove(toParent: self)
    }

    private func configureNavigationItem() {
        navigationItem.title = NSLocalizedString(
            "PROXY_CONFIGURATION_NAVIGATION_TITLE",
            tableName: "APIAccess",
            value: "Proxy configuration",
            comment: ""
        )
        navigationItem.rightBarButtonItem = testBarButton
    }

    /// Update table view controller safe area to make space for the sheet at the bottom.
    private func updateTableSafeAreaInsets() {
        let sheetHeight = sheetPresentation.isPresenting ? sheetPresentation.sheetLayoutFrame.height : 0
        var insets = contentController.additionalSafeAreaInsets
        // Prevent mutating insets if they haven't changed, in case UIKit doesn't filter duplicates.
        if insets.bottom != sheetHeight {
            insets.bottom = sheetHeight
            contentController.additionalSafeAreaInsets = insets
        }
    }

    private func onTest() {
        view.endEditing(true)
        interactor.startProxyConfigurationTest()
    }
}

extension ProxyConfigurationViewController: AccessMethodActionSheetPresentationDelegate {
    func sheetDidAdd(sheetPresentation: AccessMethodActionSheetPresentation) {}

    func sheetDidCancel(sheetPresentation: AccessMethodActionSheetPresentation) {
        interactor.cancelProxyConfigurationTest()
    }
}
