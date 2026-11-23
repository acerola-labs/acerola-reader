package br.acerola.comic

import android.app.Application
import androidx.hilt.work.HiltWorkerFactory
import androidx.work.Configuration
import br.acerola.comic.coil.PageFetcherFactory
import br.acerola.comic.common.activity.ErrorActivity
import br.acerola.comic.service.reader.ReaderProcessor
import coil.ImageLoader
import coil.ImageLoaderFactory
import coil.decode.SvgDecoder
import dagger.hilt.android.HiltAndroidApp
import javax.inject.Inject

@HiltAndroidApp
class AcerolaApplication :
    Application(),
    Configuration.Provider,
    ImageLoaderFactory {
    @Inject
    lateinit var workerFactory: HiltWorkerFactory

    override fun onCreate() {
        // First, so even a failure loading the native library lands on the error screen.
        ErrorActivity.installCrashHandler(this)
        System.loadLibrary("acerola")
        super.onCreate()
    }

    @Inject
    lateinit var readerProcessor: ReaderProcessor

    override val workManagerConfiguration: Configuration
        get() =
            Configuration
                .Builder()
                .setWorkerFactory(workerFactory)
                .build()

    override fun newImageLoader(): ImageLoader =
        ImageLoader
            .Builder(this)
            .components {
                add(SvgDecoder.Factory())
                add(PageFetcherFactory(readerProcessor))
            }.build()
}
