plugins {
    alias(libs.plugins.android.library)
}

android {
    namespace = "com.peerfora.core"
    compileSdk = 37

    defaultConfig {
        minSdk = 26
    }
}
