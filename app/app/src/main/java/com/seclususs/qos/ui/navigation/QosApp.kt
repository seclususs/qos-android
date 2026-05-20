package com.seclususs.qos.ui.navigation

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import com.seclususs.qos.ui.components.NavBar
import com.seclususs.qos.ui.features.advanced.AdvancedScreen
import com.seclususs.qos.ui.features.modules.ModulesScreen
import com.seclususs.qos.ui.features.services.ServicesScreen
import com.seclususs.qos.ui.features.settings.SettingsScreen
import kotlinx.coroutines.launch

@Composable
fun QosApp() {
    val pagerState = rememberPagerState(pageCount = { 4 })
    val coroutineScope = rememberCoroutineScope()

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        containerColor = MaterialTheme.colorScheme.background,
        bottomBar = {
            NavBar(
                selectedIndex = pagerState.currentPage, onItemSelected = { index ->
                    coroutineScope.launch {
                        pagerState.animateScrollToPage(index)
                    }
                })
        }) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(MaterialTheme.colorScheme.background)
        ) {
            HorizontalPager(
                state = pagerState, modifier = Modifier.fillMaxSize()
            ) { page ->
                when (page) {
                    0 -> ServicesScreen()
                    1 -> ModulesScreen()
                    2 -> AdvancedScreen()
                    3 -> SettingsScreen()
                }
            }
        }
    }
}