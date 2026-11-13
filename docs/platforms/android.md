# Android application

The project uses Gradle Kotlin DSL, a version catalog, Android Gradle Plugin 9.1.1, Compose, Navigation Compose, an `app` module, a `core` module, and a Glance widget module. `apps/android/gradlew` uses the Gradle 9.3.1 wrapper. The Android SDK API 37 and Build Tools 36 are expected by the scaffold.

Run `./gradlew :app:assembleDebug` from `apps/android` after installing JDK 17 or newer and the Android SDK components. `peerforaApiBaseUrl` is a Gradle property for environment-specific API configuration; the debug emulator default is `http://10.0.2.2:3000`. Live Firebase, push, Room, DataStore, WorkManager, and UniFFI integrations are not connected.
