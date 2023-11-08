//
//  MarkdownDocumentNode+AttributedString.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension MarkdownDocumentNode: AttributedMarkdown {
    func attributedString(options: MarkdownStylingOptions, applyEffect: MarkdownEffectCallback?) -> NSAttributedString {
        var isPrecededByParagraph = false

        return children.reduce(into: NSMutableAttributedString()) { partialResult, node in
            guard let transformableNode = node as? AttributedMarkdown else { return }

            defer { isPrecededByParagraph = node.isParagraph }

            // Add newline between paragraphs.
            if node.isParagraph, isPrecededByParagraph {
                partialResult.append(NSAttributedString(
                    string: "\n",
                    attributes: [.font: options.font, .paragraphStyle: options.paragraphStyle]
                ))
            }

            let attributedString = transformableNode.attributedString(options: options, applyEffect: applyEffect)
            partialResult.append(attributedString)
        }
    }
}

private extension MarkdownNode {
    var isParagraph: Bool {
        type == .paragraph
    }
}
