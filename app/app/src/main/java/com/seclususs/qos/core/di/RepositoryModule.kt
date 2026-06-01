package com.seclususs.qos.core.di

import com.seclususs.qos.data.local.datastore.AppStore
import com.seclususs.qos.data.repository.ConfigRepositoryImpl
import com.seclususs.qos.data.repository.DaemonRepositoryImpl
import com.seclususs.qos.domain.repository.ConfigRepository
import com.seclususs.qos.domain.repository.DaemonRepository
import com.seclususs.qos.domain.repository.PreferencesRepository
import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
@Suppress("unused")
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

    @Binds
    @Singleton
    abstract fun bindAppPreferencesRepository(
        appStore: AppStore
    ): PreferencesRepository
}