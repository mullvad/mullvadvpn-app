//
//  AddLocationsViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import UIKit

protocol AddLocationsViewControllerDelegate: AnyObject {
    func didUpdateSelectedLocations(locations: [RelayLocation])
    func didBack()
}

class AddLocationsViewController: UIViewController {
    private var dataSource: AddLocationsDataSource?
    private let nodes: [LocationNode]
    private let customList: CustomList

    weak var delegate: AddLocationsViewControllerDelegate?
    private let tableView: UITableView = {
        let tableView = UITableView()
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.rowHeight = 56
        tableView.indicatorStyle = .white
        tableView.accessibilityIdentifier = .addLocationsView
        tableView.allowsMultipleSelection = true
        tableView.tableHeaderView = nil
        tableView.sectionHeaderHeight = .zero
        return tableView
    }()

    private lazy var backBarButton: UIBarButtonItem = {
        let backBarButton = UIBarButtonItem(
            primaryAction: UIAction(
                image: UIImage(resource: .iconBack),
                handler: { [weak self] _ in
                    guard let self else { return }
                    delegate?.didBack()
                    navigationController?.popViewController(animated: true)
                }
            )
        )
        backBarButton.style = .done

        return backBarButton
    }()

    init(
        allLocationsNodes: [LocationNode],
        customList: CustomList
    ) {
        self.nodes = allLocationsNodes
        self.customList = customList
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        tableView.backgroundColor = view.backgroundColor
        view.backgroundColor = .secondaryColor
        navigationItem.leftBarButtonItem = backBarButton
        addConstraints()
        setUpDataSource()
    }

    private func addConstraints() {
        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }
    }

    private func setUpDataSource() {
        dataSource = AddLocationsDataSource(
            tableView: tableView,
            allLocations: nodes.copy(),
            customList: customList
        )

        dataSource?.didUpdateCustomList = { [weak self] customListLocationNode in
            guard let self else { return }
            delegate?.didUpdateSelectedLocations(
                locations: customListLocationNode.children.reduce([]) { partialResult, locationNode in
                    partialResult + locationNode.locations
                }
            )
        }
    }
}

fileprivate extension [LocationNode] {
    func copy() -> Self {
        map {
            let copy = $0.copy()
            copy.showsChildren = false
            copy.flattened.forEach { $0.showsChildren = false }
            return copy
        }
    }
}
