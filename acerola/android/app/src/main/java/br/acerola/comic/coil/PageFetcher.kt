package br.acerola.comic.coil

import android.graphics.drawable.BitmapDrawable
import android.net.Uri
import br.acerola.comic.service.reader.ReaderProcessor
import coil.ImageLoader
import coil.decode.DataSource
import coil.fetch.DrawableResult
import coil.fetch.FetchResult
import coil.fetch.Fetcher
import coil.fetch.Fetcher.Factory
import coil.request.Options

/**
 * Custom Coil fetcher that intercepts our reader's local image requests.
 *
 * **Why does this exist?**
 * Our manga/comic pages are not loose files on the file system. They are
 * embedded inside compressed archives (e.g. .cbz, .zip). Coil natively does not know how
 * to extract and load a specific page from inside a ZIP.
 *
 * **How we solve it:**
 * We created this Coil "plugin". Whenever the UI asks for an image using our custom scheme
 * (e.g. "acerola://page/comicId/chapterId/5"), Coil delegates the heavy decoding work
 * to this Fetcher, which in turn calls [ReaderProcessor].
 *
 * **Architectural advantages:**
 * 1. **UI isolation:** The UI (UI module / Jetpack Compose) does not need to know how a .cbz file
 *    is extracted. It just asks to draw a URL using Coil's `AsyncImage`.
 * 2. **OOM (Out Of Memory) prevention:** We do not load the whole Bitmap in the ViewModel (that would blow the RAM).
 *    We let Coil manage the lifecycle, background decoding and disposal of the Bitmap when
 *    the page leaves the screen.
 * 3. **Smart caching:** We take advantage of Coil's whole native multi-level memory cache system.
 *
 * @see PageFetcherFactory
 */
class PageFetcher(
    private val pageIndex: Int,
    private val readerProcessor: ReaderProcessor,
) : Fetcher {
    override suspend fun fetch(): FetchResult? {
        // ReaderProcessor goes to disk, opens the archive (e.g. ZIP/CBZ), extracts the bytes
        // of the requested page and decodes them into a Bitmap (with optimizations).
        val result = readerProcessor.loadPage(pageIndex)

        return result.fold(
            ifLeft = { null },
            ifRight = { bitmap ->
                DrawableResult(
                    drawable = BitmapDrawable(null, bitmap),
                    isSampled = false,
                    dataSource = DataSource.MEMORY,
                )
            },
        )
    }
}

/**
 * Factory responsible for registering [PageFetcher] in the app's global ImageLoader.
 *
 * **Execution flow:**
 * 1. The UI calls `AsyncImage(model = "acerola://page/1/1/5")`.
 * 2. Coil identifies the "acerola" scheme and "page" host and lands in this factory.
 * 3. The factory extracts the page index from the URL (e.g. 5) and instantiates [PageFetcher].
 */
class PageFetcherFactory(
    private val readerProcessor: ReaderProcessor,
) : Factory<Uri> {
    override fun create(
        data: Uri,
        options: Options,
        imageLoader: ImageLoader,
    ): Fetcher? {
        // Ignores requests that do not belong to our reader
        if (data.scheme != "acerola" || data.host != "page") return null

        val segments = data.pathSegments
        if (segments.size < 3) return null

        // Tries to extract the page index from the URL
        val pageIndex = segments[2].toIntOrNull() ?: return null

        return PageFetcher(pageIndex, readerProcessor)
    }
}
