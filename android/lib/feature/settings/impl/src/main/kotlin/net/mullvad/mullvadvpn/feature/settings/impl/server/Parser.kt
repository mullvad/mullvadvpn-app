package net.mullvad.mullvadvpn.feature.settings.impl.server

import kotlin.text.indexOf
import net.mullvad.mullvadvpn.feature.settings.impl.server.model.FaqBlock
import net.mullvad.mullvadvpn.feature.settings.impl.server.model.FaqItem
import net.mullvad.mullvadvpn.feature.settings.impl.server.model.RichText
import org.jsoup.nodes.Document
import org.jsoup.nodes.Element
import org.jsoup.nodes.TextNode

fun parseFaq(doc: Document): List<FaqBlock.Question> {
    val article = doc.selectFirst("main article") ?: return emptyList()

    val result = mutableListOf<FaqBlock.Question>()

    var currentTitle: String? = null
    val currentContent = mutableListOf<FaqBlock.Content>()

    for (el in article.children()) {
        when (el.tagName()) {

            "h2" -> {
                // flush previous question
                if (currentTitle != null) {
                    result += FaqBlock.Question(currentTitle, currentContent.toList())
                    currentContent.clear()
                }
                currentTitle = el.text()
            }

            "p" -> {
                currentContent += FaqBlock.Content.Paragraph(el.text())
            }

            "ul" -> {
                el.select("li").forEach { li ->
                    currentContent += FaqBlock.Content.ListItem(li.text())
                }
            }
        }
    }

    // flush last item
    if (currentTitle != null) {
        result += FaqBlock.Question(currentTitle, currentContent.toList())
    }

    return result
}

fun parseFaqFromText(doc: org.jsoup.nodes.Document): List<FaqItem> {
    val rawText = doc.body().text()

    // Split by question marks + whitespace
    val parts = rawText.split(Regex("(?<=\\?)(\\s+|$)"))

    val items = mutableListOf<FaqItem>()

    var i = 0
    while (i < parts.size) {
        val question = parts[i].trim()
        val answer = if (i + 1 < parts.size) parts[i + 1].trim() else ""
        items += FaqItem(question, answer)
        i += 2
    }

    return items
}

fun parseFaqClean(doc: org.jsoup.nodes.Document): List<FaqItem> {
    val text = doc.body().text()

    // Find where the FAQ really starts
    val faqStartIndex = Regex("\\?").find(text)?.range?.first ?: 0

    val trimmedText = text.substring(faqStartIndex)

    // Split by question/answer boundaries
    val parts = trimmedText.split(Regex("(?<=\\?).*?(?=\\S)"))

    val items = mutableListOf<FaqItem>()
    var i = 0
    while (i < parts.size - 1) {
        val question = parts[i].trim()
        val answer = parts[i + 1].trim()
        items += FaqItem(question, answer)
        i += 2
    }
    return items
}

fun parseFaqByHash(doc: Document): List<FaqItem> {
    val body = doc.body()

    // Collect all text lines
    val lines = body.text().lines()

    val items = mutableListOf<FaqItem>()
    var currentQuestion: String? = null
    val currentAnswer = StringBuilder()

    for (line in lines) {
        val trimmed = line.trim()
        if (trimmed.startsWith("#")) {
            // Save previous question/answer
            if (currentQuestion != null) {
                items += FaqItem(currentQuestion, currentAnswer.toString().trim())
                currentAnswer.clear()
            }
            // New question (remove leading #)
            currentQuestion = trimmed.removePrefix("#").trim()
        } else {
            // Append to current answer
            if (currentQuestion != null) {
                if (currentAnswer.isNotEmpty()) currentAnswer.append("\n")
                currentAnswer.append(trimmed)
            }
        }
    }

    // Add the last question
    if (currentQuestion != null) {
        items += FaqItem(currentQuestion, currentAnswer.toString().trim())
    }

    return items
}

fun parseMullvadFaq(doc: Document): List<FaqItem> {
    val items = mutableListOf<FaqItem>()

    // 1. Mullvad FAQ questions are contained within <h3> tags
    val questionHeaders = doc.select("h3")

    for (header in questionHeaders) {
        val question = header.text().trim()
        val answerBuilder = StringBuilder()

        // 2. The answer consists of all siblings (p, ul, div)
        // until we hit the next <h3> or the end of the section.
        var sibling = header.nextElementSibling()

        while (sibling != null && sibling.tagName() != "h3" && sibling.tagName() != "h2") {
            // Append the text of the sibling (paragraph, list item, etc.)
            val text = sibling.text().trim()
            if (text.isNotEmpty()) {
                if (answerBuilder.isNotEmpty()) answerBuilder.append("\n\n")
                answerBuilder.append(text)
            }
            sibling = sibling.nextElementSibling()
        }

        val answer = answerBuilder.toString().trim()

        if (question.isNotEmpty() && answer.isNotEmpty()) {
            items.add(FaqItem(question, answer))
        }
    }

    return items
}

fun parseParagraph(el: Element): FaqBlock.Content.Paragraph {
    val parts = mutableListOf<RichText.Part>()

    el.childNodes().forEach { node ->
        when (node) {
            is TextNode -> parts += RichText.Part.Text(node.text())
            is Element -> {
                if (node.tagName() == "a") {
                    parts += RichText.Part.Link(
                        node.text(),
                        node.absUrl("href")
                    )
                }
            }
        }
    }

    return FaqBlock.Content.Paragraph(parts.joinToString("") {
        when (it) {
            is RichText.Part.Text -> it.text
            is RichText.Part.Link -> it.text // simple fallback
        }
    })
}
