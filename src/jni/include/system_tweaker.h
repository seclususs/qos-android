/**
 * @brief Component responsible for applying static system configurations.
 *
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef SYSTEM_TWEAKER_H
#define SYSTEM_TWEAKER_H

/**
 * @namespace SystemTweaker
 * @brief Container for system initialization logic.
 */
namespace SystemTweaker {
    /**
     * @brief Applies the set of predefined system tweaks.
     *
     * Iterates through configured values
     * and applies them to the system.
     *
     * @note This is typically invoked once at daemon startup.
     */
    void applyAll();
}

#endif // SYSTEM_TWEAKER_H