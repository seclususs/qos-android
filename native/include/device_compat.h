/**
 * @file device_compat.h
 * @brief Device-specific compatibility checks and overrides.
 *
 * This header defines the interface for detecting specific hardware or
 * firmware configurations that require runtime adjustments to the
 * daemon's behavior to ensure stability.
 *
 * @author Seclususs
 * @see [GitHub Repository](https://github.com/seclususs/qos-android)
 */

#pragma once

namespace qos::compat {

/**
 * @brief Manages runtime compatibility adjustments for specific devices.
 */
class DeviceCompat {
public:
  /**
   * @brief Determines if the display service must be disabled for the current
   *        device.
   *
   * Checks the system properties against a list of known configurations where
   * the display service is unstable or unsupported.
   *
   * @return true if the display service must be disabled due to device
   *         compatibility constraints.
   */
  static bool should_force_disable_display();
};

} // namespace qos::compat