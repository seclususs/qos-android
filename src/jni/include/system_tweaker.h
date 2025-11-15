/**
 * @brief Applies static system-wide tweaks at daemon startup.
 * @author Seclususs
 * https://github.com/seclususs
 */

#ifndef SYSTEM_TWEAKER_H
#define SYSTEM_TWEAKER_H

/**
 * @brief Namespace containing system tweak-related functions.
 */
namespace SystemTweaker {
    /**
     * @brief Applies all predefined static system tweaks.
     */
    void applyAll();
}

#endif // SYSTEM_TWEAKER_H