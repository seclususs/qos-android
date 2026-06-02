/**
 * @file daemon.hpp
 *
 * @brief Core application entry point and daemon initialization.
 */

#pragma once

namespace qos::core
{

    /**
     * @brief Main application class for the QoS daemon.
     */
    class App
    {
        public:
        /**
         * @brief Bootstraps the daemon environment and transfers control to the runtime.
         *
         * Performs necessary privilege verification, prevents duplicate instances,
         * configures environment hardening, loads system configuration, and initializes
         * signal handling before handing control to the background runtime.
         *
         * @return EXIT_SUCCESS (0) on successful shutdown, or EXIT_FAILURE (1) if
         *         initialization fails.
         *
         * @note Execution: This is a synchronous, blocking call. It will hold the main
         *                  thread indefinitely until a graceful shutdown is explicitly
         *                  requested or a critical runtime failure occurs.
         */
        [[nodiscard]] static int bootstrap() noexcept;
    };

} // namespace qos::core