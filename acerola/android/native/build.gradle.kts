import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import java.util.Properties

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.ksp)
}

android {
    namespace = "br.acerola.comic.rust"
    compileSdk = 36
    defaultConfig {
        minSdk = 26
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
    packaging {
        jniLibs {
            pickFirsts.add("**/libjnidispatch.so")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }
    buildFeatures {
        compose = false
        buildConfig = true
    }
    kotlin {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_21)
        }
    }
    testOptions {
        unitTests.isReturnDefaultValues = true
    }
}

kotlin {
    compilerOptions {
        freeCompilerArgs.add("-Xsuppress-version-warnings")
    }
}

dependencies {
    testImplementation(libs.mockk)
    testImplementation(libs.junit)
    // Instrumented tests that really cross the FFI/JNA (real .so binary
    // compiled for the device, not a pure Rust mock) — see
    // src/androidTest/.../FileSyncFfiIntegrationTest.kt.
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.test.core.ktx)
    // `androidx.test.ext:junit` does not bring `AndroidJUnitRunner` itself — without this the
    // `testInstrumentationRunner` declared above fails with ClassNotFoundException.
    androidTestImplementation(libs.androidx.test.runner)
    implementation(libs.androidx.annotation)
    implementation(libs.kotlinx.coroutines.core)
    implementation(variantOf(libs.jna) { artifactType("aar") })
}

configure<org.jlleitschuh.gradle.ktlint.KtlintExtension> {
    filter {
        exclude { it.file.path.contains("p2p") }
        exclude { it.file.path.contains("generated") }
    }
}

tasks.withType<Test>().configureEach {
    // Ensures the lib and bindings are generated before running the unit tests
    dependsOn("generateBindings", "copyRustLib")

    systemProperty("java.library.path", "${project.projectDir}/src/main/jniLibs/arm64-v8a")
    systemProperty("jna.library.path", "${project.projectDir}/src/main/jniLibs/arm64-v8a")

    testLogging {
        events("passed", "skipped", "failed")
    }
}

val localProperties: Properties by lazy {
    val props = Properties()
    val localPropsFile = rootProject.file("local.properties")
    if (localPropsFile.exists()) {
        localPropsFile.inputStream().use { props.load(it) }
    }
    props
}

val cargo: String =
    project.findProperty("cargo.dir") as? String
        ?: System.getenv("CARGO")
        ?: localProperties.getProperty("cargo.dir")
        ?: "cargo"

// The whole Rust/UniFFI build (compile, generate bindings, copy .so) is delegated to
// `cargo make` (see `rust/Makefile.toml`) instead of several `Exec` tasks calling `cargo`
// directly — the cross-platform copy/mkdir logic lives in cargo-make's duckscript instead
// of scattered `ifeq ($(OS),Windows_NT)`, and adding a new build variant (e.g.
// x86_64 for the emulator) is just a new task in Makefile.toml, without touching Gradle.
val cargoMake: String =
    project.findProperty("cargo.make.dir") as? String
        ?: System.getenv("CARGO_MAKE")
        ?: localProperties.getProperty("cargo.make.dir")
        ?: "cargo-make"

// Validates that the resolved `cargo-make` exists BEFORE any task tries to use it — without
// this, the failure shown is a raw "native process not found" error, without saying
// it was cargo-make that was missing nor how to configure a custom path.
val validateCargoMake by
    tasks.registering {
        doFirst {
            // `cargo-make.exe` expects `make` as its first argument (that is how
            // `cargo` internally invokes any `cargo-XXX` plugin) — calling just
            // `cargo-make --version` directly breaks with a confusing parser error.
            val found =
                runCatching {
                    ProcessBuilder(cargoMake, "make", "--version").redirectErrorStream(true).start().waitFor() == 0
                }.getOrDefault(false)

            if (!found) {
                throw GradleException(
                    "cargo-make not found (looked in '$cargoMake'). Install it with " +
                        "`cargo install cargo-make`, or set the path via the project property " +
                        "`cargo.make.dir`, the CARGO_MAKE env var, or `cargo.make.dir` in " +
                        "local.properties.",
                )
            }
        }
    }

tasks.register<Exec>("buildRust") {
    dependsOn(validateCargoMake)
    workingDir = file("rust")
    commandLine(cargoMake, "make", "--makefile", "makefile.toml", "build-rust")
}

tasks.register<Exec>("generateBindings") {
    dependsOn("buildRust")
    workingDir = file("rust")
    commandLine(cargoMake, "make", "--makefile", "makefile.toml", "generate-bindings")
}

tasks.register<Exec>("copyRustLib") {
    dependsOn("buildRust")
    workingDir = file("rust")
    commandLine(cargoMake, "make", "--makefile", "makefile.toml", "copy-libs")
}

// Not part of `preBuild`/`connectedAndroidTest` by default — only needed to run
// `connectedAndroidTest` on an EMULATOR (x86_64), which does not load the default arm64-v8a .so.
// Run it manually first: `./gradlew :native:buildRustEmulator`.
tasks.register<Exec>("buildRustEmulator") {
    dependsOn(validateCargoMake)
    workingDir = file("rust")
    commandLine(cargoMake, "make", "--makefile", "makefile.toml", "build-emulator")
}

tasks.named("preBuild") {
    dependsOn("generateBindings", "copyRustLib")
}

tasks.configureEach {
    val isJniTask = name.startsWith("merge") && name.contains("JniLib")
    val isTestTask = name.contains("AndroidTest") || name.startsWith("connected")

    if (isJniTask || isTestTask) {
        dependsOn("generateBindings", "copyRustLib")
    }
}
