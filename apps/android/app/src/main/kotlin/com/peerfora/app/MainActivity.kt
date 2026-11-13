package com.peerfora.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.peerfora.core.ApiEndpointConfiguration

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val apiConfiguration = ApiEndpointConfiguration(BuildConfig.PEERFORA_API_BASE_URL)
        setContent {
            PeerforaAndroidApplication(apiConfiguration.baseUrl)
        }
    }
}

@Composable
private fun PeerforaAndroidApplication(apiBaseUrl: String) {
    val navigationController = rememberNavController()
    val homePageTitle = if (apiBaseUrl.isBlank()) {
        "Peerfora API address is not configured"
    } else {
        "Peerfora Android foundation is ready"
    }

    MaterialTheme {
        Surface {
            NavHost(
                navController = navigationController,
                startDestination = "home",
            ) {
                composable("home") {
                    PeerforaPlaceholderScreen(homePageTitle)
                }
                composable("communities") {
                    PeerforaPlaceholderScreen("Communities")
                }
                composable("inbox") {
                    PeerforaPlaceholderScreen("Inbox")
                }
                composable("settings") {
                    PeerforaPlaceholderScreen("Settings")
                }
            }
        }
    }
}

@Composable
private fun PeerforaPlaceholderScreen(title: String) {
    androidx.compose.material3.Text(text = title)
}
