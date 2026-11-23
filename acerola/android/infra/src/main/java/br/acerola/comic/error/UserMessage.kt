package br.acerola.comic.error

import br.acerola.comic.type.UiText

interface UserMessage {
    val uiMessage: UiText

    // Every error type (ChapterError, NetworkError, etc.) is a UserMessage and is always a failure —
    // only Raw needs to be able to represent a success message, which is why the default here
    // covers the rest without requiring every error sealed interface to implement it.
    val isSuccess: Boolean get() = false

    data class Raw(
        override val uiMessage: UiText,
        override val isSuccess: Boolean = false,
    ) : UserMessage {
        constructor(message: String) : this(UiText.DynamicString(message))
    }
}
