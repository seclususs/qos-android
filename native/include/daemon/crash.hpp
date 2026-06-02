/**
 * @file crash.hpp
 *
 * @brief Emergency signal management.
 */

#pragma once

namespace qos::core
{

    /**
     * @brief Manager for handling fatal system signals.
     */
    class Handler
    {
        public:
        /**
         * @brief Installs emergency signal traps for fatal execution errors.
         *
         * Intercepts fatal system signals (e.g., SIGSEGV, SIGABRT) to print a
         * critical pre-death warning to standard error before delegating
         * the termination back to the default OS crash handler.
         */
        static void arm() noexcept;
    };

} // namespace qos::core