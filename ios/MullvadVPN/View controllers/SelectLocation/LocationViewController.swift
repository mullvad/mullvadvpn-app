//
//  LocationViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

protocol LocationViewControllerDelegate: AnyObject {
    func navigateToCustomLists(nodes: [LocationNode])
    func navigateToDaitaSettings()
    func didSelectRelays(relays: UserSelectedRelays)
    func didUpdateFilter(filter: RelayFilter)
}

final class LocationViewController: UIViewController {
    private let searchBar = UISearchBar()
    private let tableView = UITableView(frame: .zero, style: .grouped)
    private let topContentView = UIStackView()
    private let filterView = RelayFilterView()
    private var daitaInfoView: UIView?
    private var dataSource: LocationDataSource?
    private var relaysWithLocation: LocationRelays?
    private var filter = RelayFilter()
    private var selectedRelays: RelaySelection
    private var shouldFilterDaita: Bool
    weak var delegate: LocationViewControllerDelegate?
    var customListRepository: CustomListRepositoryProtocol

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var filterViewShouldBeHidden: Bool {
        !shouldFilterDaita && (filter.ownership == .any) && (filter.providers == .any)
    }

    init(
        customListRepository: CustomListRepositoryProtocol,
        selectedRelays: RelaySelection,
        shouldFilterDaita: Bool
    ) {
        self.customListRepository = customListRepository
        self.selectedRelays = selectedRelays
        self.shouldFilterDaita = shouldFilterDaita

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        view.accessibilityIdentifier = .selectLocationView
        view.backgroundColor = .secondaryColor

        setUpDataSource()
        setUpTableView()
        setUpTopContent()

        view.addConstrainedSubviews([topContentView, tableView]) {
            topContentView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))

            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: topContentView.bottomAnchor, constant: 8)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        dataSource?.scrollToSelectedRelay()
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        tableView.flashScrollIndicators()
    }

    // MARK: - Public

    func setRelaysWithLocation(_ relaysWithLocation: LocationRelays, filter: RelayFilter) {
        self.relaysWithLocation = relaysWithLocation
        self.filter = filter

        if filterViewShouldBeHidden {
            filterView.isHidden = true
        } else {
            filterView.isHidden = false
            filterView.setFilter(filter)
        }

        dataSource?.setRelays(relaysWithLocation, selectedRelays: selectedRelays)
    }

    func setShouldFilterDaita(_ shouldFilterDaita: Bool) {
        self.shouldFilterDaita = shouldFilterDaita

        filterView.isHidden = filterViewShouldBeHidden
        filterView.setDaita(shouldFilterDaita)
    }

    func refreshCustomLists() {
        dataSource?.refreshCustomLists(selectedRelays: selectedRelays)
    }

    func setSelectedRelays(_ selectedRelays: RelaySelection) {
        self.selectedRelays = selectedRelays
        dataSource?.setSelectedRelays(selectedRelays)
    }

    func addDaitaInfoView() {
        guard daitaInfoView == nil else { return }

        let infoLabel = UILabel()
        infoLabel.numberOfLines = 0

        let infoTextParagraphStyle = NSMutableParagraphStyle()
        infoTextParagraphStyle.lineSpacing = 1.3
        infoTextParagraphStyle.alignment = .center

        infoLabel.attributedText = NSAttributedString(
            string: NSLocalizedString(
                "SELECT_LOCATION_DAITA_INFO",
                tableName: "SelectLocation",
                value: "DAITA overrides Multihop. To use Multihop, please turn on Direct only in the DAITA settings.",
                comment: ""
            ),
            attributes: [
                .font: UIFont.systemFont(ofSize: 15),
                .foregroundColor: UIColor.white,
                .paragraphStyle: infoTextParagraphStyle,
            ]
        )

        let settingsButton = AppButton(style: .default)
        settingsButton.setTitle(NSLocalizedString(
            "SELECT_LOCATION_DAITA_BUTTON",
            tableName: "SelectLocation",
            value: "Go to DAITA settings",
            comment: ""
        ), for: .normal)

        settingsButton.addTarget(self, action: #selector(didPressDaitaSettingsButton), for: .touchUpInside)

        let contentView = UIStackView(arrangedSubviews: [infoLabel, settingsButton])
        contentView.axis = .vertical
        contentView.spacing = 32
        contentView.isLayoutMarginsRelativeArrangement = true
        contentView.layoutMargins = UIMetrics.contentInsets

        let daitaInfoView = UIView()
        self.daitaInfoView = daitaInfoView
        daitaInfoView.backgroundColor = .secondaryColor

        daitaInfoView.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.init([.leading(0), .trailing(0)]))
            contentView.bottomAnchor.constraint(equalTo: daitaInfoView.centerYAnchor)
        }

        view.addConstrainedSubviews([daitaInfoView]) {
            daitaInfoView.pinEdgesToSuperview(.all().excluding(.top))
            daitaInfoView.topAnchor.constraint(equalTo: tableView.topAnchor)
        }
    }

    func removeDaitaInfoView() {
        daitaInfoView?.removeFromSuperview()
        daitaInfoView = nil
    }

    @objc private func didPressDaitaSettingsButton() {
        delegate?.navigateToDaitaSettings()
    }

    // MARK: - Private

    private func setUpDataSource() {
        dataSource = LocationDataSource(
            tableView: tableView,
            allLocations: AllLocationDataSource(),
            customLists: CustomListsDataSource(repository: customListRepository)
        )

        dataSource?.didSelectRelayLocations = { [weak self] relays in
            self?.delegate?.didSelectRelays(relays: relays)
        }

        dataSource?.didTapEditCustomLists = { [weak self] in
            guard let self else { return }

            if let relaysWithLocation {
                let allLocationDataSource = AllLocationDataSource()
                allLocationDataSource.reload(relaysWithLocation)
                delegate?.navigateToCustomLists(nodes: allLocationDataSource.nodes)
            }
        }

        if let relaysWithLocation {
            dataSource?.setRelays(relaysWithLocation, selectedRelays: selectedRelays)
        }
    }

    private func setUpTableView() {
        tableView.backgroundColor = view.backgroundColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.rowHeight = 56
        tableView.sectionHeaderHeight = 56
        tableView.indicatorStyle = .white
        tableView.keyboardDismissMode = .onDrag
        tableView.accessibilityIdentifier = .selectLocationTableView
    }

    private func setUpTopContent() {
        topContentView.axis = .vertical
        topContentView.addArrangedSubview(filterView)
        topContentView.addArrangedSubview(searchBar)

        filterView.isHidden = filterViewShouldBeHidden
        filterView.setDaita(shouldFilterDaita)

        filterView.didUpdateFilter = { [weak self] in
            self?.delegate?.didUpdateFilter(filter: $0)
        }

        setUpSearchBar()
    }

    private func setUpSearchBar() {
        searchBar.delegate = self
        searchBar.searchBarStyle = .minimal
        searchBar.layer.cornerRadius = 8
        searchBar.clipsToBounds = true
        searchBar.placeholder = NSLocalizedString(
            "SEARCHBAR_PLACEHOLDER",
            tableName: "SelectLocation",
            value: "Search for...",
            comment: ""
        )
        searchBar.searchTextField.accessibilityIdentifier = .selectLocationSearchTextField

        UITextField.SearchTextFieldAppearance.inactive.apply(to: searchBar)
    }
}

extension LocationViewController: UISearchBarDelegate {
    func searchBar(_ searchBar: UISearchBar, textDidChange searchText: String) {
        dataSource?.filterRelays(by: searchText)
    }

    func searchBarTextDidBeginEditing(_ searchBar: UISearchBar) {
        UITextField.SearchTextFieldAppearance.active.apply(to: searchBar)
    }

    func searchBarTextDidEndEditing(_ searchBar: UISearchBar) {
        UITextField.SearchTextFieldAppearance.inactive.apply(to: searchBar)
    }
}
