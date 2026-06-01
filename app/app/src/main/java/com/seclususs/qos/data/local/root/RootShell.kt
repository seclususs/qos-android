package com.seclususs.qos.data.local.root

import com.topjohnwu.superuser.Shell
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class RootShell @Inject constructor() {

    val isRootAvailable: Boolean
        get() = Shell.getShell().isRoot

    fun execute(command: String): String? {
        if (!isRootAvailable) return null
        val result = Shell.cmd(command).exec()
        return result.out.joinToString("\n").trim().ifEmpty { null }
    }

    fun executeSilently(command: String): Boolean {
        if (!isRootAvailable) return false
        return Shell.cmd(command).exec().isSuccess
    }

    fun readFile(path: String): String? = execute("cat $path")

    fun writeFile(path: String, content: String): Boolean {
        if (!isRootAvailable) return false
        val escapedContent = content.replace("'", "'\\''")
        return executeSilently("echo '$escapedContent' > $path")
    }
}