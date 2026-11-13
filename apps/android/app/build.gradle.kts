plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "com.peerfora.app"
    compileSdk = 37

    defaultConfig {
        applicationId = "com.peerfora.student"
        minSdk = 26
        targetSdk = 37
        versionCode = 1
        versionName = "0.1.0"
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildTypes {
        debug {
            val apiBaseUrl = providers.gradleProperty("peerforaApiBaseUrl")
                .orElse("http://10.0.2.2:3000")
                .get()
            buildConfigField("String", "PEERFORA_API_BASE_URL", "\"$apiBaseUrl\"")
        }
        release {
            isMinifyEnabled = false
            val apiBaseUrl = providers.gradleProperty("peerforaApiBaseUrl").orElse("").get()
            buildConfigField("String", "PEERFORA_API_BASE_URL", "\"$apiBaseUrl\"")
        }
    }
}

dependencies {
    implementation(project(":core"))
    implementation(project(":widgets"))
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.adaptive)
    implementation(libs.navigation.compose)
    debugImplementation(libs.androidx.compose.ui.tooling)
}
