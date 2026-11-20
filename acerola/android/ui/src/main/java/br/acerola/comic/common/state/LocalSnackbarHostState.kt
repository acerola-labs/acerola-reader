package br.acerola.comic.common.state
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.staticCompositionLocalOf

val LocalSnackbarHostState =
    staticCompositionLocalOf<SnackbarHostState> {
        error(
            "LocalSnackbarHostState was not provided. Wrap the content with CompositionLocalProvider(LocalSnackbarHostState provides snackbarHostState).",
        )
    }
