package com.seclususs.qos.core.di

import com.seclususs.qos.data.repository.ConfigRepositoryImpl
import com.seclususs.qos.data.repository.DaemonRepositoryImpl
import com.seclususs.qos.domain.repository.ConfigRepository
import com.seclususs.qos.domain.repository.DaemonRepository
import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class RepositoryModule {

    @Binds
    @Singleton
    abstract fun bindDaemonRepository(
        daemonRepositoryImpl: DaemonRepositoryImpl
    ): DaemonRepository

    @Binds
    @Singleton
    abstract fun bindConfigRepository(
        configRepositoryImpl: ConfigRepositoryImpl
    ): ConfigRepository
}