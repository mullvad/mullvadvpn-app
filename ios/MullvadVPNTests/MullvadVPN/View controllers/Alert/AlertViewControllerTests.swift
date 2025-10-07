//
//  AlertViewControllerTests.swift
//  MullvadVPNTests
//
//  Created by Test Suite on 2025-10-07.
//

import Testing
import UIKit

@testable import MullvadVPN

@Suite("AlertViewController")
@MainActor
struct AlertViewControllerTests {
    @Test("Defaults accessibility identifier to alertContainerView")
    func testDefaultAccessibilityIdentifier() {
        let presentation = AlertPresentation(
            id: "default-id",
            buttons: []
        )
        let vc = makeVC(presentation)
        #expect(vc.view.accessibilityIdentifier == AccessibilityIdentifier.alertContainerView.asString)
    }

    @Test("Uses explicit accessibility identifier when provided")
    func testExplicitAccessibilityIdentifier() {
        let presentation = AlertPresentation(
            id: "explicit-id",
            accessibilityIdentifier: .settingsContainerView,
            buttons: []
        )
        let vc = makeVC(presentation)
        #expect(vc.view.accessibilityIdentifier == AccessibilityIdentifier.settingsContainerView.asString)
    }

    @Test("Header label has accessibility identifier")
    func testHeaderAccessibility() {
        let presentation = AlertPresentation(
            id: "header-id",
            header: "Header",
            buttons: []
        )
        let vc = makeVC(presentation)
        let header = findView(withAccessibilityId: AccessibilityIdentifier.alertTitle.asString, in: vc.view)
        #expect(header != nil)
    }

    @Test("Spinner icon is present in view hierarchy")
    func testSpinnerIcon() {
        let presentation = AlertPresentation(
            id: "spinner-id",
            icon: .spinner,
            buttons: []
        )
        let vc = makeVC(presentation)
        let spinner = findSubview(ofType: SpinnerActivityIndicatorView.self, in: vc.view)
        #expect(spinner != nil)
    }

    @Test("Button tap invokes onDismiss and action handler")
    func testButtonTapInvokesHandlers() {
        var dismissed = false
        var tapped = false
        let action = AlertAction(title: "OK", style: .default, accessibilityId: .alertOkButton, handler: { tapped = true })
        let presentation = AlertPresentation(id: "tap-id", title: "Title", buttons: [action])
        let vc = makeVC(presentation)
        vc.onDismiss = { dismissed = true }

        let buttons: [AppButton] = findAllSubviews(ofType: AppButton.self, in: vc.view)
        let okButton = buttons.first { $0.title(for: .normal) == "OK" }
        #expect(okButton != nil)
        okButton?.sendActions(for: .touchUpInside)

        #expect(tapped)
        #expect(dismissed)
    }

    @Test("Layout cycles produce non-zero scroll content size")
    func testLayoutProducesScrollContent() {
        let longMessage = String(repeating: "Long message. ", count: 100)
        let presentation = AlertPresentation(id: "layout-id", title: "Title", message: longMessage, buttons: [])
        let vc = makeVC(presentation)

        // Force a couple of layout passes
        for _ in 0..<2 {
            vc.view.setNeedsLayout()
            vc.view.layoutIfNeeded()
        }
        let scroll = findSubview(ofType: UIScrollView.self, in: vc.view)
        #expect(scroll != nil)
        #expect((scroll?.contentSize.height ?? 0) > 0)
    }

    @Test("AlertActionStyle maps to AppButton style")
    func testButtonStyleMapping() {
        let actions = [
            AlertAction(title: "Default", style: .default, accessibilityId: nil, handler: nil),
            AlertAction(title: "Delete", style: .destructive, accessibilityId: nil, handler: nil),
        ]
        let presentation = AlertPresentation(id: "styles-id", title: "Title", buttons: actions)
        let vc = makeVC(presentation)

        let buttons: [AppButton] = findAllSubviews(ofType: AppButton.self, in: vc.view)
        let defaultButton = buttons.first { $0.title(for: .normal) == "Default" }
        let deleteButton = buttons.first { $0.title(for: .normal) == "Delete" }
        #expect(defaultButton != nil)
        #expect(deleteButton != nil)

        #expect(defaultButton?.style == .default)
        #expect(deleteButton?.style == .danger)
    }
}

// MARK: - Helpers
@MainActor
private func makeVC(_ presentation: AlertPresentation) -> AlertViewController {
    let vc = AlertViewController(presentation: presentation)
    vc.loadViewIfNeeded()
    vc.view.setNeedsLayout()
    vc.view.layoutIfNeeded()
    return vc
}

private func findSubview<T: UIView>(ofType: T.Type, in root: UIView) -> T? {
    if let view = root as? T { return view }
    for sub in root.subviews {
        if let v: T = findSubview(ofType: ofType, in: sub) { return v }
    }
    return nil
}

private func findAllSubviews<T: UIView>(ofType: T.Type, in root: UIView) -> [T] {
    var result: [T] = []
    if let v = root as? T { result.append(v) }
    for sub in root.subviews {
        result.append(contentsOf: findAllSubviews(ofType: ofType, in: sub))
    }
    return result
}

private func findView(withAccessibilityId id: String, in root: UIView) -> UIView? {
    if let ident = (root as? UIAccessibilityIdentification)?.accessibilityIdentifier, ident == id { return root }
    for sub in root.subviews {
        if let found = findView(withAccessibilityId: id, in: sub) { return found }
    }
    return nil
}