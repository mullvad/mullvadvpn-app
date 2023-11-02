//
//  AddAccessMethodViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 08/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

/// The view controller providing the interface for adding new access method.
class AddAccessMethodViewController: UIViewController, UITableViewDelegate {
    private let interactor: AddAccessMethodInteractorProtocol
    private var validationError: AccessMethodValidationError?
    private let viewModelSubject: CurrentValueSubject<AccessMethodViewModel, Never>
    private var cancellables = Set<AnyCancellable>()
    private var dataSource: UITableViewDiffableDataSource<
        AddAccessMethodSectionIdentifier,
        AddAccessMethodItemIdentifier
    >?
    private lazy var cancelBarButton: UIBarButtonItem = {
        UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.onCancel()
            })
        )
    }()

    private lazy var addBarButton: UIBarButtonItem = {
        let barButtonItem = UIBarButtonItem(
            title: NSLocalizedString("ADD_NAVIGATION_BUTTON", tableName: "APIAccess", value: "Add", comment: ""),
            primaryAction: UIAction { [weak self] _ in
                self?.onAdd()
            }
        )
        barButtonItem.style = .done
        return barButtonItem
    }()

    private lazy var sheetPresentation: AccessMethodActionSheetPresentation = {
        let sheetPresentation = AccessMethodActionSheetPresentation()
        sheetPresentation.delegate = self
        return sheetPresentation
    }()

    private let contentController = UITableViewController(style: .insetGrouped)
    private var tableView: UITableView { contentController.tableView }

    weak var delegate: AddAccessMethodViewControllerDelegate?

    init(subject: CurrentValueSubject<AccessMethodViewModel, Never>, interactor: AddAccessMethodInteractorProtocol) {
        self.viewModelSubject = subject
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

    private func dequeueCell(at indexPath: IndexPath, for itemIdentifier: AddAccessMethodItemIdentifier)
        -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: itemIdentifier.cellIdentifier, for: indexPath)

        if let cell = cell as? DynamicBackgroundConfiguration {
            cell.setAutoAdaptingBackgroundConfiguration(.mullvadListGroupedCell(), selectionType: .dimmed)
        }

        switch itemIdentifier {
        case .name:
            configureName(cell, itemIdentifier: itemIdentifier)
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
            let section = SocksSectionHandler(tableStyle: tableView.style, subject: viewModelSubject)
            section.configure(cell, itemIdentifier: socksItemIdentifier)

        case let .shadowsocks(shadowsocksItemIdentifier):
            let section = ShadowsocksSectionHandler(tableStyle: tableView.style, subject: viewModelSubject)
            section.configure(cell, itemIdentifier: shadowsocksItemIdentifier)
        }
    }

    private func configureName(_ cell: UITableViewCell, itemIdentifier: AddAccessMethodItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .optional)
        contentConfiguration.textFieldProperties = .withAutoResignAndDoneReturnKey()
        contentConfiguration.inputText = viewModelSubject.value.name
        contentConfiguration.editingEvents.onChange = viewModelSubject.bindTextAction(to: \.name)
        cell.contentConfiguration = contentConfiguration
    }

    private func configureProtocol(_ cell: UITableViewCell, itemIdentifier: AddAccessMethodItemIdentifier) {
        var contentConfiguration = UIListContentConfiguration.mullvadValueCell(tableStyle: tableView.style)
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.secondaryText = viewModelSubject.value.method.localizedDescription
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

        viewModelSubject.withPreviousValue().sink { [weak self] previousValue, newValue in
            self?.viewModelDidChange(previousValue: previousValue, newValue: newValue)
        }
        .store(in: &cancellables)
    }

    private func viewModelDidChange(previousValue: AccessMethodViewModel?, newValue: AccessMethodViewModel) {
        let animated = view.window != nil
        let previousValidationError = validationError

        validate()
        updateBarButtons(newValue: newValue)
        updateSheet(previousValue: previousValue, newValue: newValue, animated: animated)
        updateModalPresentation(newValue: newValue)
        updateDataSource(
            previousValue: previousValue,
            newValue: newValue,
            previousValidationError: previousValidationError,
            newValidationError: validationError,
            animated: animated
        )
    }

    private func updateSheet(previousValue: AccessMethodViewModel?, newValue: AccessMethodViewModel, animated: Bool) {
        guard previousValue?.testingStatus != newValue.testingStatus else { return }

        switch newValue.testingStatus {
        case .initial:
            sheetPresentation.hide(animated: animated)

        case .inProgress, .failed, .succeeded:
            var presentationConfiguration = AccessMethodActionSheetPresentationConfiguration()
            presentationConfiguration.sheetConfiguration.context = .addNew
            presentationConfiguration.sheetConfiguration.contentConfiguration.status = newValue.testingStatus
                .sheetStatus
            sheetPresentation.configuration = presentationConfiguration

            sheetPresentation.show(in: view, animated: animated)
        }
    }

    private func updateDataSource(
        previousValue: AccessMethodViewModel?,
        newValue: AccessMethodViewModel,
        previousValidationError: AccessMethodValidationError?,
        newValidationError: AccessMethodValidationError?,
        animated: Bool
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<AddAccessMethodSectionIdentifier, AddAccessMethodItemIdentifier>()

        snapshot.appendSections([.name, .protocol])
        snapshot.appendItems([.name], toSection: .name)

        snapshot.appendItems([.protocol], toSection: .protocol)
        // Reconfigure the protocol item on the access method change.
        if let previousValue, previousValue.method != newValue.method {
            snapshot.reconfigureOrReloadItems([.protocol])
        }

        if newValue.method.hasProxyConfiguration {
            snapshot.appendSections([.proxyConfiguration])
        }

        switch newValue.method {
        case .direct, .bridges:
            break

        case .shadowsocks:
            snapshot.appendItems(AddAccessMethodItemIdentifier.allShadowsocksItems, toSection: .proxyConfiguration)
            // Reconfigure cipher item on change.
            if let previousValue, previousValue.shadowsocks.cipher != newValue.shadowsocks.cipher {
                snapshot.reconfigureOrReloadItems([.proxyConfiguration(.shadowsocks(.cipher))])
            }

        case .socks5:
            snapshot.appendItems(
                AddAccessMethodItemIdentifier.allSocksItems(authenticate: newValue.socks.authenticate),
                toSection: .proxyConfiguration
            )
        }

        dataSource?.apply(snapshot, animatingDifferences: animated)
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
        navigationItem.prompt = NSLocalizedString(
            "ADD_METHOD_NAVIGATION_PROMPT",
            tableName: "APIAccess",
            value: "The app will test the method before adding it.",
            comment: ""
        )
        navigationItem.title = NSLocalizedString(
            "ADD_METHOD_NAVIGATION_TITLE",
            tableName: "APIAccess",
            value: "Add access method",
            comment: ""
        )
        navigationItem.leftBarButtonItem = cancelBarButton
        navigationItem.rightBarButtonItem = addBarButton
    }

    private func validate() {
        let validationResult = Result { try viewModelSubject.value.validate() }
        validationError = validationResult.error as? AccessMethodValidationError
    }

    private func updateBarButtons(newValue: AccessMethodViewModel) {
        addBarButton.isEnabled = newValue.testingStatus == .initial && validationError == nil
        cancelBarButton.isEnabled = newValue.testingStatus == .initial
    }

    private func updateModalPresentation(newValue: AccessMethodViewModel) {
        // Prevent swipe gesture when testing or when the sheet offers user actions.
        isModalInPresentation = newValue.testingStatus != .initial
    }

    private func onAdd() {
        view.endEditing(true)

        interactor.startProxyConfigurationTest { [weak self] succeeded in
            if succeeded {
                self?.addMethodAndNotifyDelegate(afterDelay: true)
            }
        }
    }

    private func onCancel() {
        view.endEditing(true)
        interactor.cancelProxyConfigurationTest()

        delegate?.controllerDidCancel(self)
    }

    /// Tells interactor to add the access method and then notifies the delegate which then dismisses the view controller.
    /// - Parameter afterDelay: whether to add a short delay before calling the delegate.
    private func addMethodAndNotifyDelegate(afterDelay: Bool) {
        interactor.addMethod()

        guard afterDelay else {
            sendControllerDidAdd()
            return
        }

        // Add a short delay to let user see the sheet with successful status before the delegate dismisses the view
        // controller.
        DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) { [weak self] in
            self?.sendControllerDidAdd()
        }
    }

    private func sendControllerDidAdd() {
        delegate?.controllerDidAdd(self)
    }
}

extension AddAccessMethodViewController: AccessMethodActionSheetPresentationDelegate {
    func sheetDidAdd(sheetPresentation: AccessMethodActionSheetPresentation) {
        addMethodAndNotifyDelegate(afterDelay: false)
    }

    func sheetDidCancel(sheetPresentation: AccessMethodActionSheetPresentation) {
        interactor.cancelProxyConfigurationTest()
    }
}
