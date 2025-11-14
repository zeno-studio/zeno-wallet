package com.zenowallet

import android.os.Bundle
import app.tauri.PluginManager
import app.tauri.TauriActivity
import com.zenowallet.photo.PhotoPlugin

class MainActivity : TauriActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // 注册插件
        PluginManager.registerPlugin(PhotoPlugin::class.java)
    }
}
