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
    private let minPanelHeight: CGFloat = 120
    private var maxPanelHeight: CGFloat = 500
    private var panelHeight: CGFloat = 350
    private var safeTop: CGFloat = 60
    private var safeBottom: CGFloat = 40
    private var dragStartY: CGFloat = 0
    private var resizeStartHeight: CGFloat = 0

    private let interactor: LogViewInteractor
    private let topHandleView = UIView()
    private let topHandleBar = UIView()
    private let bottomHandleView = UIView()
    private let bottomHandleBar = UIView()
    private let pauseButton = IncreasedHitButton()
    private let clearButton = IncreasedHitButton()
    private let exportButton = IncreasedHitButton()
    private let searchField = UITextField()
    private let tableView = UITableView(frame: .zero, style: .plain)
    private let processButton = IncreasedHitButton()
    private let includeButton = IncreasedHitButton()
    private let excludeButton = IncreasedHitButton()

    private var entries: [InAppLogEntry] = []
    private var filteredEntries: [InAppLogEntry] = []
    private var pausedEntries: [InAppLogEntry] = []
    private var searchText = ""
    private var logsArePaused: Bool = false
    private var selectedProcess: InAppLogEntry.Process?
    private var includedLabels: Set<String> = []
    private var excludedLabels: Set<String> = []

    var onExportLogs: ((String) -> Void)?

    init(interactor: LogViewInteractor) {
        self.interactor = interactor

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

        interactor.didAddEntry = { [weak self] entry in
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

        topHandleBar.backgroundColor = .white.withAlphaComponent(0.5)
        topHandleBar.layer.cornerRadius = 2

        addConstrainedSubviews([topHandleView]) {
            topHandleView.pinEdgesToSuperview(.all().excluding(.bottom))
            topHandleView.heightAnchor.constraint(equalToConstant: 28)
        }

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

        let trailingContainer = UIStackView(arrangedSubviews: [exportButton, clearButton])
        trailingContainer.spacing = 4

        addConstrainedSubviews([leadingContainer, trailingContainer]) {
            leadingContainer.pinEdgeToSuperview(.leading(8))
            leadingContainer.centerYAnchor.constraint(equalTo: topHandleBar.centerYAnchor)

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

        let processImage = UIImage(systemName: "app.connected.to.app.below.fill")?
            .withConfiguration(UIImage.SymbolConfiguration(pointSize: 12, weight: .medium))
        processButton.setImage(processImage, for: .normal)
        processButton.tintColor = .white.withAlphaComponent(0.5)
        processButton.showsMenuAsPrimaryAction = true
        processButton.menu = buildProcessMenu()

        let includeImage = UIImage(systemName: "plus.circle")?
            .withConfiguration(UIImage.SymbolConfiguration(pointSize: 12, weight: .medium))
        includeButton.setImage(includeImage, for: .normal)
        includeButton.tintColor = .white.withAlphaComponent(0.5)
        includeButton.showsMenuAsPrimaryAction = true
        includeButton.menu = buildIncludeMenu()

        let excludeImage = UIImage(systemName: "minus.circle")?
            .withConfiguration(UIImage.SymbolConfiguration(pointSize: 12, weight: .medium))
        excludeButton.setImage(excludeImage, for: .normal)
        excludeButton.tintColor = .white.withAlphaComponent(0.5)
        excludeButton.showsMenuAsPrimaryAction = true
        excludeButton.menu = buildExcludeMenu()

        addConstrainedSubviews([searchField, processButton, includeButton, excludeButton]) {
            searchField.pinEdgeToSuperview(.leading(8))
            searchField.topAnchor.constraint(equalTo: topHandleView.bottomAnchor)
            searchField.heightAnchor.constraint(equalToConstant: 28)

            processButton.leadingAnchor.constraint(equalTo: searchField.trailingAnchor, constant: 4)
            processButton.centerYAnchor.constraint(equalTo: searchField.centerYAnchor)
            processButton.widthAnchor.constraint(equalToConstant: 28)

            includeButton.leadingAnchor.constraint(equalTo: processButton.trailingAnchor)
            includeButton.centerYAnchor.constraint(equalTo: searchField.centerYAnchor)
            includeButton.widthAnchor.constraint(equalToConstant: 28)

            excludeButton.pinEdgeToSuperview(.trailing(8))
            excludeButton.leadingAnchor.constraint(equalTo: includeButton.trailingAnchor)
            excludeButton.centerYAnchor.constraint(equalTo: searchField.centerYAnchor)
            excludeButton.widthAnchor.constraint(equalToConstant: 28)
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

        bottomHandleBar.backgroundColor = .white.withAlphaComponent(0.5)
        bottomHandleBar.layer.cornerRadius = 2

        addConstrainedSubviews([bottomHandleView]) {
            bottomHandleView.topAnchor.constraint(equalTo: tableView.bottomAnchor)
            bottomHandleView.pinEdgesToSuperview(.all().excluding(.top))
            bottomHandleView.heightAnchor.constraint(equalToConstant: 20)
        }

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
        rebuildFilterMenuIfNeeded()

        for entry in entries where matchesFilter(entry) {
            filteredEntries.append(entry)
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
        let matchesSearch = searchText.isEmpty || entry.description.localizedCaseInsensitiveContains(searchText)
        let matchesProcess = selectedProcess == nil || entry.process == selectedProcess
        let isIncluded = includedLabels.isEmpty || includedLabels.contains(entry.label)
        let notExcluded = !excludedLabels.contains(entry.label)

        return matchesSearch && matchesProcess && isIncluded && notExcluded
    }

    private func buildProcessMenu() -> UIMenu {
        let allAction = UIAction(title: "All", state: selectedProcess == nil ? .on : .off) { [weak self] _ in
            guard let self else { return }

            selectedProcess = nil
            processButton.menu = buildProcessMenu()
            applyFilter()
        }

        let processActions = InAppLogEntry.Process.allCases.map { process in
            UIAction(
                title: process.rawValue,
                state: selectedProcess == process ? .on : .off
            ) { [weak self] _ in
                guard let self else { return }

                selectedProcess = process
                processButton.menu = buildProcessMenu()
                applyFilter()
            }
        }

        return UIMenu(title: "Process", children: [allAction] + processActions)
    }

    private func buildIncludeMenu() -> UIMenu {
        let uniqueLabels = Set(entries.map { $0.label }).sorted()

        let actions = uniqueLabels.map { label in
            UIAction(title: label, state: includedLabels.contains(label) ? .on : .off) { [weak self] _ in
                guard let self else { return }

                if includedLabels.contains(label) {
                    includedLabels.remove(label)
                } else {
                    includedLabels.insert(label)
                }

                includeButton.menu = buildIncludeMenu()
                applyFilter()
            }
        }

        return UIMenu(title: "Include only", children: actions)
    }

    private func buildExcludeMenu() -> UIMenu {
        let uniqueLabels = Set(entries.map { $0.label }).sorted()

        let actions = uniqueLabels.map { label in
            UIAction(title: label, state: excludedLabels.contains(label) ? .on : .off) { [weak self] _ in
                guard let self else { return }

                if excludedLabels.contains(label) {
                    excludedLabels.remove(label)
                } else {
                    excludedLabels.insert(label)
                }

                excludeButton.menu = buildExcludeMenu()
                applyFilter()
            }
        }

        return UIMenu(title: "Exclude", children: actions)
    }

    private func rebuildFilterMenuIfNeeded() {
        let uniqueLabels = Set(entries.map { $0.label })
        let currentLabels = Set((excludeButton.menu?.children.compactMap { ($0 as? UIAction)?.title }) ?? [])

        if uniqueLabels != currentLabels {
            includeButton.menu = buildIncludeMenu()
            excludeButton.menu = buildExcludeMenu()
        }
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
        onExportLogs?(filteredEntries.map { $0.description }.joinedParagraphs())
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
            NSAttributedString(
                string: entry.label,
                attributes: [
                    .foregroundColor: entry.process == .app ? UIColor.systemYellow : UIColor.systemCyan,
                    .font: font,
                ]
            )
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
