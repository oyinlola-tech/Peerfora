package com.peerfora.widgets

import androidx.compose.runtime.Composable
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver
import androidx.glance.appwidget.provideContent
import androidx.glance.layout.Column
import androidx.glance.layout.padding
import androidx.glance.text.Text
import androidx.compose.ui.unit.dp

class PeerforaHomeWidget : GlanceAppWidget() {
    override suspend fun provideGlance(context: android.content.Context, id: GlanceId) {
        provideContent {
            PeerforaHomeWidgetContent()
        }
    }
}

class PeerforaHomeWidgetReceiver : GlanceAppWidgetReceiver() {
    override val glanceAppWidget = PeerforaHomeWidget()
}

@Composable
private fun PeerforaHomeWidgetContent() {
    Column(modifier = GlanceModifier.padding(16.dp)) {
        Text(text = "Peerfora")
        Text(text = "Open the app to explore your student community.")
    }
}
