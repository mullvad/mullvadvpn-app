//
//  LogView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

class LogView: UIView, UITableViewDataSource, UITableViewDelegate {
    private let panelHeight: CGFloat = 230
    private let safeTop: CGFloat = 60
    private let safeBottom: CGFloat = 40

    private let viewModel: LogViewModel
    private let tableView = UITableView(frame: .zero, style: .plain)
    private let handleView = UIView()

    private var entries: [String] = []
    private var dragStartY: CGFloat = 0

    // Snaps to top, middle and bottom.
    private lazy var snapYPositions: [CGFloat] = {
        let screenHeight = UIScreen.main.bounds.height
        return [safeTop, (screenHeight - panelHeight) / 2, screenHeight - panelHeight - safeBottom]
    }()

    init(viewModel: LogViewModel) {
        self.viewModel = viewModel

        super.init(frame: .zero)

        setUp()
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setUp() {
        frame = CGRect(
            x: 8,
            y: snapYPositions.first!,
            width: UIScreen.main.bounds.width - 16,
            height: panelHeight
        )

        backgroundColor = .black.withAlphaComponent(0.7)
        layer.cornerRadius = 12
        clipsToBounds = true

        viewModel.didAddEntry = addEntry

        setUpHandle()
        setUpTableView()

        addGestureRecognizer(UIPanGestureRecognizer(target: self, action: #selector(handlePan(_:))))
    }

    private func setUpHandle() {
        handleView.backgroundColor = .white.withAlphaComponent(0.5)
        handleView.layer.cornerRadius = 2

        addConstrainedSubviews([handleView]) {
            handleView.pinEdgeToSuperview(.top(8))
            handleView.centerXAnchor.constraint(equalTo: centerXAnchor)
            handleView.widthAnchor.constraint(equalToConstant: 36)
            handleView.heightAnchor.constraint(equalToConstant: 4)
        }
    }

    private func setUpTableView() {
        tableView.dataSource = self
        tableView.delegate = self
        tableView.register(UITableViewCell.self, forCellReuseIdentifier: "LogCell")

        tableView.backgroundColor = .clear
        tableView.separatorStyle = .none
        tableView.showsVerticalScrollIndicator = true
        tableView.indicatorStyle = .white
        tableView.verticalScrollIndicatorInsets.right = 2

        addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: handleView.bottomAnchor, constant: 8)
        }
    }

    private func addEntry(_ entry: String) {
        entries.append(entry)

        let indexPath = IndexPath(row: entries.count - 1, section: 0)
        tableView.insertRows(at: [indexPath], with: .none)
        tableView.scrollToRow(at: indexPath, at: .bottom, animated: true)
    }

    // MARK: - Dragging

    @objc private func handlePan(_ gesture: UIPanGestureRecognizer) {
        switch gesture.state {
        case .began:
            dragStartY = frame.origin.y
        case .changed:
            let translation = gesture.translation(in: superview).y
            frame.origin.y = dragStartY + translation
        case .ended, .cancelled:
            let currentY = frame.origin.y
            let targetY = snapYPositions.min(by: { abs($0 - currentY) < abs($1 - currentY) }) ?? safeTop

            UIView.animate(
                withDuration: 0.35,
                delay: 0,
                usingSpringWithDamping: 0.8,
                initialSpringVelocity: 0,
                options: .curveEaseOut
            ) {
                self.frame.origin.y = targetY
            }
        default:
            break
        }
    }

    // MARK: - UITableViewDataSource

    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        entries.count
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "LogCell", for: indexPath)

        var config = cell.defaultContentConfiguration()
        config.text = entries[indexPath.row]
        config.textProperties.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        config.textProperties.color = .white

        cell.contentConfiguration = config
        cell.backgroundColor = .clear
        cell.selectionStyle = .none

        return cell
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldShowMenuForRowAt indexPath: IndexPath) -> Bool {
        true
    }

    func tableView(
        _ tableView: UITableView,
        canPerformAction action: Selector,
        forRowAt indexPath: IndexPath,
        withSender sender: Any?
    ) -> Bool {
        action == #selector(copy(_:))
    }

    func tableView(
        _ tableView: UITableView,
        performAction action: Selector,
        forRowAt indexPath: IndexPath,
        withSender sender: Any?
    ) {
        UIPasteboard.general.string = entries[indexPath.row]
    }
}
