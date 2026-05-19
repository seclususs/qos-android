package com.seclususs.qos.data.repository

import com.seclususs.qos.core.di.IoDispatcher
import com.seclususs.qos.data.local.root.ConfigParser
import com.seclususs.qos.data.local.root.RootShell
import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.domain.repository.ConfigRepository
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ConfigRepositoryImpl @Inject constructor(
    private val rootShell: RootShell,
    private val configParser: ConfigParser,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher
) : ConfigRepository {

    private val configPath = "/data/adb/modules/sys_qos/config.ini"

    override suspend fun checkConfigExists(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("ls $configPath")
    }

    override suspend fun getConfig(): QosConfig = withContext(ioDispatcher) {
        val rawText = rootShell.readFile(configPath)
        if (rawText.isNullOrBlank()) {
            QosConfig()
        } else {
            configParser.parse(rawText)
        }
    }

    override suspend fun updateConfig(config: QosConfig): Boolean = withContext(ioDispatcher) {
        val newRawText = configParser.serialize(config)
        rootShell.writeFile(configPath, newRawText)
    }
}