//
//  LogView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import UIKit

class LogView: UIView {
    private var maxPanelHeight: CGFloat = 500
    private let minPanelHeight: CGFloat = 120
    private var panelHeight: CGFloat = 230
    private var safeTop: CGFloat = 60
    private var safeBottom: CGFloat = 40
    private var dragStartY: CGFloat = 0
    private var resizeStartHeight: CGFloat = 0

    private let viewModel: LogViewModel
    private let topHandleView = UIView()
    private let topHandleBar = UIView()
    private let bottomHandleView = UIView()
    private let bottomHandleBar = UIView()
    private let pauseButton = IncreasedHitButton()
    private let clearButton = IncreasedHitButton()
    private let exportButton = IncreasedHitButton()
    private let searchField = UITextField()
    private let tableView = UITableView(frame: .zero, style: .plain)

    private var entries: [InAppLogEntry] = []
    private var filteredEntries: [InAppLogEntry] = []
    private var pausedEntries: [InAppLogEntry] = []
    private var searchText = ""
    private var logsArePaused: Bool = false

    var onExportLogs: ((String) -> Void)?

    init(viewModel: LogViewModel) {
        self.viewModel = viewModel

        super.init(frame: .zero)

        setUp()
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()
        guard let insets = window?.safeAreaInsets else { return }

        safeTop = insets.top
        safeBottom = insets.bottom
        maxPanelHeight = UIScreen.main.bounds.height - safeTop - safeBottom

        frame.origin.y = safeTop + 100
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setUp() {
        frame = CGRect(
            x: 8,
            y: safeTop,
            width: UIScreen.main.bounds.width - 16,
            height: panelHeight
        )

        backgroundColor = .black.withAlphaComponent(0.7)
        layer.cornerRadius = 12
        clipsToBounds = true

        viewModel.didAddEntry = { [weak self] entry in
            self?.addEntries([entry])
        }

        setUpTopHandle()
        setUpButtons()
        setUpSearchField()
        setUpTableView()
        setUpBottomHandle()
    }

    private func setUpTopHandle() {
        topHandleView.backgroundColor = .clear

        addConstrainedSubviews([topHandleView]) {
            topHandleView.pinEdgesToSuperview(.all().excluding(.bottom))
            topHandleView.heightAnchor.constraint(equalToConstant: 28)
        }

        topHandleBar.backgroundColor = .white.withAlphaComponent(0.5)
        topHandleBar.layer.cornerRadius = 2

        topHandleView.addConstrainedSubviews([topHandleBar]) {
            topHandleBar.centerXAnchor.constraint(equalTo: topHandleView.centerXAnchor)
            topHandleBar.centerYAnchor.constraint(equalTo: topHandleView.centerYAnchor)
            topHandleBar.widthAnchor.constraint(equalToConstant: 36)
            topHandleBar.heightAnchor.constraint(equalToConstant: 4)
        }

        let panGesture = UIPanGestureRecognizer(target: self, action: #selector(handlePan(_:)))
        topHandleView.addGestureRecognizer(panGesture)
    }

    private func setUpButtons() {
        pauseButton.setTitle("[pause]", for: .normal)
        pauseButton.titleLabel?.font = .preferredFont(forTextStyle: .caption2)
        pauseButton.titleLabel?.tintColor = .white.withAlphaComponent(0.5)
        pauseButton.addTarget(self, action: #selector(handlePauseButton), for: .touchUpInside)

        exportButton.setTitle("[export]", for: .normal)
        exportButton.titleLabel?.font = .preferredFont(forTextStyle: .caption2)
        exportButton.titleLabel?.tintColor = .white.withAlphaComponent(0.5)
        exportButton.addTarget(self, action: #selector(handleExportButton), for: .touchUpInside)

        clearButton.setTitle("[clear]", for: .normal)
        clearButton.titleLabel?.font = .preferredFont(forTextStyle: .caption2)
        clearButton.titleLabel?.tintColor = .white.withAlphaComponent(0.5)
        clearButton.addTarget(self, action: #selector(handleClearButton), for: .touchUpInside)

        let leadingContainer = UIStackView(arrangedSubviews: [pauseButton])
        leadingContainer.spacing = 4

        addConstrainedSubviews([leadingContainer]) {
            leadingContainer.pinEdgeToSuperview(.leading(8))
            leadingContainer.centerYAnchor.constraint(equalTo: topHandleBar.centerYAnchor)
        }

        let trailingContainer = UIStackView(arrangedSubviews: [exportButton, clearButton])
        trailingContainer.spacing = 4

        addConstrainedSubviews([trailingContainer]) {
            trailingContainer.pinEdgeToSuperview(.trailing(8))
            trailingContainer.centerYAnchor.constraint(equalTo: topHandleBar.centerYAnchor)
        }
    }

    private func setUpSearchField() {
        let placeholder = "Search logs..."
        searchField.placeholder = placeholder
        searchField.attributedPlaceholder = NSAttributedString(
            string: placeholder,
            attributes: [.foregroundColor: UIColor.white.withAlphaComponent(0.4)]
        )

        searchField.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        searchField.textColor = .white
        searchField.backgroundColor = .white.withAlphaComponent(0.1)
        searchField.layer.cornerRadius = 6
        searchField.leftView = UIView(frame: CGRect(x: 0, y: 0, width: 8, height: 0))
        searchField.leftViewMode = .always
        searchField.returnKeyType = .search
        searchField.autocorrectionType = .no
        searchField.autocapitalizationType = .none

        let clearImage = UIImage(systemName: "xmark.circle.fill")?
            .withConfiguration(UIImage.SymbolConfiguration(pointSize: 10))
        let clearButton = UIButton(type: .system)
        clearButton.setImage(clearImage, for: .normal)
        clearButton.tintColor = .white.withAlphaComponent(0.5)
        clearButton.addTarget(self, action: #selector(clearSearchField), for: .touchUpInside)
        clearButton.sizeToFit()

        let clearButtonContainer = UIView(
            frame: CGRect(x: 0, y: 0, width: clearButton.frame.width + 16, height: clearButton.frame.height)
        )
        clearButton.frame.origin = .init(x: 8, y: 0)
        clearButtonContainer.addSubview(clearButton)

        searchField.rightView = clearButtonContainer
        searchField.rightViewMode = .whileEditing

        searchField.addTarget(self, action: #selector(searchTextChanged), for: .editingChanged)
        searchField.delegate = self

        addConstrainedSubviews([searchField]) {
            searchField.topAnchor.constraint(equalTo: topHandleView.bottomAnchor)
            searchField.pinEdgesToSuperview(.init([.leading(8), .trailing(8)]))
            searchField.heightAnchor.constraint(equalToConstant: 28)
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
        tableView.keyboardDismissMode = .onDrag

        addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview(.all().excluding(.top).excluding(.bottom))
            tableView.topAnchor.constraint(equalTo: searchField.bottomAnchor, constant: 8)
        }
    }

    private func setUpBottomHandle() {
        bottomHandleView.backgroundColor = .clear

        addConstrainedSubviews([bottomHandleView]) {
            bottomHandleView.topAnchor.constraint(equalTo: tableView.bottomAnchor)
            bottomHandleView.pinEdgesToSuperview(.all().excluding(.top))
            bottomHandleView.heightAnchor.constraint(equalToConstant: 20)
        }

        bottomHandleBar.backgroundColor = .white.withAlphaComponent(0.5)
        bottomHandleBar.layer.cornerRadius = 2

        bottomHandleView.addConstrainedSubviews([bottomHandleBar]) {
            bottomHandleBar.centerXAnchor.constraint(equalTo: bottomHandleView.centerXAnchor)
            bottomHandleBar.centerYAnchor.constraint(equalTo: bottomHandleView.centerYAnchor)
            bottomHandleBar.widthAnchor.constraint(equalToConstant: 36)
            bottomHandleBar.heightAnchor.constraint(equalToConstant: 4)
        }

        let resizeGesture = UIPanGestureRecognizer(target: self, action: #selector(handleResize(_:)))
        bottomHandleView.addGestureRecognizer(resizeGesture)
    }

    private func addEntries(_ entries: [InAppLogEntry]) {
        guard !entries.isEmpty else { return }

        guard !logsArePaused else {
            pausedEntries.append(contentsOf: entries)
            return
        }

        self.entries.append(contentsOf: entries)

        for entry in entries where matchesFilter(entry) {
            filteredEntries.append(contentsOf: entries)
        }

        tableView.reloadData()
        scrollToBottom()
    }

    private func clearEntries() {
        entries.removeAll()
        filteredEntries.removeAll()
        pausedEntries.removeAll()

        tableView.reloadData()
        scrollToBottom()
    }

    private func applyFilter() {
        filteredEntries = entries.filter { matchesFilter($0) }

        tableView.reloadData()
        scrollToBottom()
    }

    private func matchesFilter(_ entry: InAppLogEntry) -> Bool {
        searchText.isEmpty || entry.description.localizedCaseInsensitiveContains(searchText)
    }

    private func scrollToBottom() {
        guard !filteredEntries.isEmpty else { return }

        let indexPath = IndexPath(row: filteredEntries.count - 1, section: 0)
        tableView.scrollToRow(at: indexPath, at: .bottom, animated: true)
    }

    // MARK: - Actions

    @objc private func handlePauseButton(_ sender: UIButton) {
        logsArePaused.toggle()
        pauseButton.setTitle(logsArePaused ? "[resume]" : "[pause]", for: .normal)

        addEntries(pausedEntries)
        pausedEntries.removeAll()
    }

    @objc private func handleClearButton(_ sender: UIButton) {
        clearEntries()
    }

    @objc private func handleExportButton() {
        onExportLogs?(filteredEntries.description)
    }

    @objc private func searchTextChanged() {
        searchText = searchField.text ?? ""
        applyFilter()
    }

    @objc private func clearSearchField() {
        searchField.text = ""
        searchTextChanged()
    }

    // MARK: - Resizing

    @objc private func handleResize(_ gesture: UIPanGestureRecognizer) {
        switch gesture.state {
        case .began:
            resizeStartHeight = frame.height
        case .changed:
            let translation = gesture.translation(in: superview).y
            let newHeight = min(max(resizeStartHeight + translation, minPanelHeight), maxPanelHeight)
            frame.size.height = newHeight
            panelHeight = newHeight
        case .ended, .cancelled:
            scrollToBottom()
        default:
            break
        }
    }

    // MARK: - Dragging

    @objc private func handlePan(_ gesture: UIPanGestureRecognizer) {
        let screenHeight = UIScreen.main.bounds.height
        let maxY = screenHeight - safeBottom

        switch gesture.state {
        case .began:
            dragStartY = frame.origin.y
        case .changed:
            let translation = gesture.translation(in: superview).y
            frame.origin.y = min(max(dragStartY + translation, safeTop), maxY)
        case .ended, .cancelled:
            let velocity = gesture.velocity(in: superview).y
            let projectedY = frame.origin.y + velocity * 0.08
            let targetY = min(max(projectedY, safeTop), maxY)

            UIView.animate(
                withDuration: 0.4,
                delay: 0,
                usingSpringWithDamping: 0.75,
                initialSpringVelocity: abs(velocity) / max(abs(targetY - frame.origin.y), 1),
                options: .curveEaseOut
            ) {
                self.frame.origin.y = targetY
            }
        default:
            break
        }
    }
}

extension LogView: UITableViewDataSource {
    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        filteredEntries.count
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "LogCell", for: indexPath)

        let entry = filteredEntries[indexPath.row]
        let font = UIFont.monospacedSystemFont(ofSize: 11, weight: .regular)

        let attributed = NSMutableAttributedString()
        attributed.append(
            NSAttributedString(string: entry.timestamp, attributes: [.foregroundColor: UIColor.lightGray, .font: font])
        )
        attributed.append(NSAttributedString(string: " "))
        attributed.append(
            NSAttributedString(string: entry.label, attributes: [.foregroundColor: UIColor.systemYellow, .font: font])
        )
        attributed.append(NSAttributedString(string: "\n"))
        attributed.append(
            NSAttributedString(string: entry.message, attributes: [.foregroundColor: UIColor.white, .font: font])
        )

        var config = cell.defaultContentConfiguration()
        config.attributedText = attributed

        cell.contentConfiguration = config
        cell.backgroundColor = .clear
        cell.selectionStyle = .none

        return cell
    }
}

extension LogView: UITableViewDelegate {
    func tableView(
        _ tableView: UITableView,
        contextMenuConfigurationForRowAt indexPath: IndexPath,
        point: CGPoint
    ) -> UIContextMenuConfiguration? {
        let entry = filteredEntries[indexPath.row]

        return UIContextMenuConfiguration(identifier: indexPath as NSIndexPath, previewProvider: nil) { [weak self] _ in
            let copy = UIAction(title: "Copy", image: UIImage(systemName: "doc.on.doc")) { _ in
                UIPasteboard.general.string = entry.description
            }

            let share = UIAction(title: "Share", image: UIImage(systemName: "square.and.arrow.up")) { _ in
                self?.onExportLogs?(entry.description)
            }

            return UIMenu(children: [copy, share])
        }
    }

    func tableView(
        _ tableView: UITableView,
        previewForHighlightingContextMenuWithConfiguration configuration: UIContextMenuConfiguration
    ) -> UITargetedPreview? {
        clearTargetedPreview(for: configuration, in: tableView)
    }

    func tableView(
        _ tableView: UITableView,
        previewForDismissingContextMenuWithConfiguration configuration: UIContextMenuConfiguration
    ) -> UITargetedPreview? {
        clearTargetedPreview(for: configuration, in: tableView)
    }

    private func clearTargetedPreview(
        for configuration: UIContextMenuConfiguration,
        in tableView: UITableView
    ) -> UITargetedPreview? {
        guard
            let indexPath = configuration.identifier as? NSIndexPath,
            let cell = tableView.cellForRow(at: indexPath as IndexPath)
        else {
            return nil
        }

        let parameters = UIPreviewParameters()
        parameters.backgroundColor = .clear
        parameters.shadowPath = UIBezierPath()

        return UITargetedPreview(view: cell, parameters: parameters)
    }
}

extension LogView: UITextFieldDelegate {
    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        textField.resignFirstResponder()
        return true
    }
}
