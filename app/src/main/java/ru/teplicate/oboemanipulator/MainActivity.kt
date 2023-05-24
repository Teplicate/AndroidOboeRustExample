package ru.teplicate.oboemanipulator

import android.content.Context
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import ru.teplicate.oboemanipulator.ui.theme.OboeManipulatorTheme
import java.io.File

class MainActivity : ComponentActivity() {

    init {
        System.loadLibrary("oboelib")
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val file = moveAssetsToAppDir(this)
        setContent {
            OboeManipulatorTheme {
                val nativeLayer = remember {
                    NativeLayer()
                }

                // A surface container using the 'background' color from the theme
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    Box(
                        modifier = Modifier.fillMaxWidth(),
                        contentAlignment = Alignment.Center
                    ) {
                        Button(onClick = { nativeLayer.playWav(file.absolutePath) }) {
                            Text(text = "Play sound")
                        }
                    }
                }
            }
        }
    }
}

private fun moveAssetsToAppDir(context: Context): File {
    return context.resources.openRawResource(R.raw.drum_kick)
        .use { iss ->
            val appDir = context.filesDir
            val asset = File(appDir, "assets")

            if (!asset.exists())
                asset.mkdir()

            val wavFile = File(asset, "drum_kick.wav")

            if (!wavFile.exists()) {
                wavFile.createNewFile()
            }

            wavFile.outputStream().use { os ->
                iss.copyTo(os)
            }

            wavFile
        }
}